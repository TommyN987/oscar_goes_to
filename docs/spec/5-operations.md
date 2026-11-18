# Package 5: operations

Part of [the specification](README.md). Levels and source codes are defined there.

Done when one restore from backup has been rehearsed.

| # | Level | Requirement | Done when | Source |
|---|---|---|---|---|
| 5.1 | MUST | An encrypted database dump goes daily, from 21 January to 31 March, to a Google Cloud bucket in a US region | During the season, a dump from yesterday exists in the bucket | U, R |
| 5.1.1 | MUST | The decryption key is held in Tommy's password manager, outside CI | The rehearsal in 5.2 uses that copy | U |
| 5.2 | MUST | A restore has been rehearsed | A dump restored into a scratch database has matching row counts | C |
| 5.3 | MUST | A ceremony-night runbook covers pre-checks, entering and correcting results, rollback, and the paper fallback | The document exists and was followed in the rehearsal | C |
| 5.4 | SHOULD | A dress rehearsal runs a fake season end to end before 7 March | A fake season reaches Complete with a correct leaderboard | C |
| 5.5 | SHOULD | An uptime monitor pings liveness only | The monitor's target is the liveness path | R, C |
| 5.6 | MUST | A pre-season checklist each December verifies every account, the domain renewal and the Google client, and re-enables and verifies the backup job | The checklist exists with a dated first run | R, U |
| 5.7 | MAY | An alert emails the admin on a raised error rate | A forced error triggers one email | T |

## Notes

- **The bucket is free only in a US region.**
  The dump holds EU players' emails, so it is encrypted before upload.
- **The key must live outside CI.**
  GitHub secrets cannot be read back, so a key kept only there makes every dump unrestorable.
- **There is no off-season dump.**
  GitHub disables scheduled workflows after 60 days without repository activity (T, probable), so a weekly job would stop silently.
  Off-season data barely changes, so the last in-season dump is the off-season backup.
- **Neon's six-hour history is not a backup.**
  The app is idle nine months a year, so a loss would be noticed far later than six hours.
- **Backups cannot go to GitHub artifacts.**
  The repo is public and the dump contains emails.
- **Pinging readiness would exhaust Neon's free compute.**
  A monitor that touches the database keeps it awake all month:
  about 180 compute-hours at the smallest size against 100 free.
  Hence liveness only in 5.5.
