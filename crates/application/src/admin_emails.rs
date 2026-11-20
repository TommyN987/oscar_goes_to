//! Who is an admin: the accounts named in the configuration (spec rule 1.6.3).

use std::fmt;

/// The email addresses of the admins, compared without regard to case.
///
/// Its `Debug` output hides the addresses, so they cannot reach a log or a span (spec rule 2.7.6).
#[derive(Clone, PartialEq, Eq)]
pub struct AdminEmails(Vec<String>);

impl AdminEmails {
    /// No admin at all.
    pub const fn none() -> Self {
        Self(Vec::new())
    }

    /// Reads a comma-separated list. Blank entries are ignored.
    pub fn parse(list: &str) -> Self {
        Self(
            list.split(',')
                .map(|email| email.trim().to_lowercase())
                .filter(|email| !email.is_empty())
                .collect(),
        )
    }

    pub fn contains(&self, email: &str) -> bool {
        let email = email.to_lowercase();
        self.0.contains(&email)
    }
}

impl fmt::Debug for AdminEmails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AdminEmails(<{} configured>)", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_email_on_the_list_is_an_admin_whatever_its_case() {
        let admins = AdminEmails::parse("tommy@example.com, Second.Admin@Example.com");

        assert!(admins.contains("tommy@example.com"));
        assert!(admins.contains("TOMMY@example.com"));
        assert!(admins.contains("second.admin@example.com"));
        assert!(!admins.contains("player@example.com"));
    }

    #[test]
    fn an_empty_list_names_no_admin() {
        assert!(!AdminEmails::none().contains("tommy@example.com"));
        assert!(!AdminEmails::parse("").contains(""));
        assert!(!AdminEmails::parse(" , ,").contains(""));
    }

    #[test]
    fn rule_2_7_6_debug_output_hides_the_emails() {
        let admins = AdminEmails::parse("tommy@example.com,second@example.com");

        assert_eq!(format!("{admins:?}"), "AdminEmails(<2 configured>)");
    }
}
