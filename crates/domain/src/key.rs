//! Author-chosen keys: the stable names of films, categories and nominees (spec rule 4.1.2).
//!
//! A key is made of lowercase letters, digits and hyphens. It is written in the seed file and ends up
//! in URLs, so it stays within what both take without quoting or escaping.

/// A string that cannot be a key.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not a key: a key is made of lowercase letters, digits and hyphens")]
pub struct InvalidKey(String);

/// Returns `text` as the content of a key, or refuses it.
/// A key has at least one character, and each is a lowercase ASCII letter, a digit or a hyphen.
pub(crate) fn checked(text: &str) -> Result<String, InvalidKey> {
    let allowed = |character: char| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-';
    if !text.is_empty() && text.chars().all(allowed) {
        Ok(text.to_owned())
    } else {
        Err(InvalidKey(text.to_owned()))
    }
}

/// Defines a key type: a string that passed [`checked`], with `parse`, `as_str` and `Display`.
/// The three kinds of key are separate types, so one cannot be passed for another.
macro_rules! key_type {
    ($(#[$attribute:meta])* $name:ident) => {
        $(#[$attribute])*
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Accepts `key` if it is made of lowercase letters, digits and hyphens.
            pub fn parse(key: &str) -> Result<Self, $crate::key::InvalidKey> {
                $crate::key::checked(key).map(Self)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}
pub(crate) use key_type;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_4_1_2_a_key_is_made_of_lowercase_letters_digits_and_hyphens() {
        for key in ["picture", "a", "7", "international-feature", "category-24", "x-2-y"] {
            assert_eq!(checked(key), Ok(key.to_owned()), "{key}");
        }
    }

    #[test]
    fn rule_4_1_2_anything_else_is_not_a_key() {
        let not_keys = [
            "",
            " ",
            "Picture",
            "best picture",
            "best_picture",
            "picture/2027",
            "pictur\u{00e9}",
            "picture\n",
        ];

        for text in not_keys {
            assert_eq!(checked(text), Err(InvalidKey(text.to_owned())), "{text:?}");
        }
    }

    #[test]
    fn rule_4_1_4_a_refused_key_is_named_with_the_rule_it_breaks() {
        assert_eq!(
            checked("Best Picture").unwrap_err().to_string(),
            "`Best Picture` is not a key: a key is made of lowercase letters, digits and hyphens"
        );
    }
}
