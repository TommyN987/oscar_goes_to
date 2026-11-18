#!/usr/bin/env python3
"""PreToolUse hook for Bash: Claude may read git, and Tommy alone changes it.

Every git subcommand is denied unless it is on the read-only list below.
That covers creating branches, staging, committing, pushing, merging, resetting and stashing.

The check is conservative, not a shell parser: text that merely looks like a git
command, such as a quoted "git commit", is denied too.
"""

import json
import re
import sys

# Subcommands that never change the repository, whatever their arguments.
READ_ONLY = {
    "blame", "cat-file", "check-attr", "check-ignore", "cherry", "count-objects", "describe",
    "diff", "diff-tree", "for-each-ref", "grep", "help", "log", "ls-files", "ls-remote",
    "ls-tree", "merge-base", "name-rev", "range-diff", "rev-list", "rev-parse", "shortlog",
    "show", "show-branch", "show-ref", "status", "var", "verify-commit", "verify-tag",
    "version", "whatchanged",
}

# Subcommands that only read when their first positional argument is one of these.
# None stands for "no positional argument at all".
READS_WITH_FIRST_ARGUMENT = {
    "remote": {None, "show", "get-url"},
    "stash": {"list", "show"},
    "worktree": {"list"},
    "reflog": {None, "show", "exists"},
    "config": {"get", "list"},
}

# `git branch` and `git tag` list by default and write when given a name or one of these flags.
WRITING_FLAGS = {
    "branch": {"-d", "-D", "-m", "-M", "-c", "-C", "-f", "-u", "--delete", "--move", "--copy",
               "--force", "--set-upstream-to", "--unset-upstream", "--edit-description", "--track"},
    "tag": {"-d", "-a", "-s", "-u", "-f", "-m", "-F", "-e", "--delete", "--annotate", "--sign",
            "--force", "--message", "--file", "--edit"},
}
LISTING_FLAGS = {"-l", "--list", "--contains", "--no-contains", "--merged", "--no-merged",
                 "--points-at", "--sort", "--format"}

# Options of `git` itself that take their value as the next word.
GLOBAL_OPTIONS_WITH_VALUE = {"-C", "-c", "--git-dir", "--work-tree", "--namespace", "--config-env"}

# Words that may stand before a command without being the command.
COMMAND_PREFIXES = {"sudo", "command", "env", "exec", "time", "nohup", "xargs", "then", "do", "else", "!"}

COMMAND_SEPARATORS = re.compile(r"\n|;|&&|\|\||\||&|\(|\)|`|\{|\}")
ENVIRONMENT_ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
REDIRECTION = re.compile(r"^\d*(>>?|<)")


def unquoted(word):
    return word.strip("\"'")


def is_git(word):
    return word == "git" or word.endswith("/git")


def git_invocations(command):
    """Yields the words after `git` for everything in `command` that looks like a git invocation."""
    for segment in COMMAND_SEPARATORS.split(command):
        words = segment.split()
        at_command_position = True
        for index, word in enumerate(words):
            starts_quoted_text = word[:1] in "\"'"
            if is_git(unquoted(word)) and (at_command_position or starts_quoted_text):
                yield words[index + 1:]
                break
            if at_command_position and not (ENVIRONMENT_ASSIGNMENT.match(word) or word in COMMAND_PREFIXES):
                at_command_position = False


def without_redirections(words):
    kept = []
    skip_next = False
    for word in words:
        if skip_next:
            skip_next = False
        elif REDIRECTION.match(word):
            skip_next = REDIRECTION.sub("", word) == ""
        else:
            kept.append(word)
    return kept


def subcommand_and_arguments(words):
    """Splits the words after `git` into its subcommand and that subcommand's arguments."""
    words = [unquoted(word) for word in without_redirections(words)]
    index = 0
    while index < len(words) and words[index].startswith("-"):
        index += 2 if words[index] in GLOBAL_OPTIONS_WITH_VALUE else 1
    if index >= len(words):
        return None, []
    return words[index], words[index + 1:]


def only_reads(subcommand, arguments):
    flags = {argument.split("=")[0] for argument in arguments if argument.startswith("-")}
    positionals = [argument for argument in arguments if not argument.startswith("-")]
    first_positional = positionals[0] if positionals else None

    if subcommand is None or subcommand in READ_ONLY:
        return True
    if subcommand == "fetch":
        # A refspec such as `main:main` would move a local branch.
        return not any(":" in positional for positional in positionals)
    if subcommand == "config" and flags & {"--get", "--get-all", "--get-regexp", "--list", "-l"}:
        return True
    if subcommand in READS_WITH_FIRST_ARGUMENT:
        return first_positional in READS_WITH_FIRST_ARGUMENT[subcommand]
    if subcommand in WRITING_FLAGS:
        if flags & WRITING_FLAGS[subcommand]:
            return False
        return not positionals or bool(flags & LISTING_FLAGS)
    return False


def main():
    command = json.load(sys.stdin).get("tool_input", {}).get("command", "")
    for words in git_invocations(command):
        subcommand, arguments = subcommand_and_arguments(words)
        if not only_reads(subcommand, arguments):
            reason = (
                f"`git {subcommand}` could change the repository, and Tommy makes every git change himself. "
                "Read-only git commands are allowed. Do not look for another way to do this: "
                "tell Tommy which command to run and why."
            )
            json.dump(
                {
                    "hookSpecificOutput": {
                        "hookEventName": "PreToolUse",
                        "permissionDecision": "deny",
                        "permissionDecisionReason": reason,
                    }
                },
                sys.stdout,
            )
            return


if __name__ == "__main__":
    main()
