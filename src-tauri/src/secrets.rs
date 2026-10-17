//! The secrets todai keeps, such as Notion's personal access tokens: in the macOS Keychain, never in a database or a
//! backup.

use std::{future::Future, path::PathBuf, process::Stdio, time::Duration};

use tokio::process::Command;

use crate::{
    errors::{Result, TodaiError},
    process::{self, RunError},
};

/// Where secrets are kept, each under an account (e.g. a workspace's ID): the Keychain, or a fake one in tests.
pub trait SecretStore: Sync {
    /// The secret of `account`, if any. To keep in memory only, and never log.
    fn get(&self, account: &str) -> impl Future<Output = Result<Option<String>>> + Send;

    /// Keeps `secret` as the one of `account`, instead of any other one.
    fn set(&self, account: &str, secret: &str) -> impl Future<Output = Result<()>> + Send;

    /// Forgets the secret of `account`, returning whether there was one.
    fn delete(&self, account: &str) -> impl Future<Output = Result<bool>> + Send;
}

const SECURITY: &str = "/usr/bin/security";

/// How long the `security` tool may take: it only reads or writes the login keychain.
const TIMEOUT: Duration = Duration::from_secs(10);

/// What `security` exits with when there's no such item: `errSecItemNotFound`'s low byte.
const ITEM_NOT_FOUND: i32 = 44;

/// The user's login keychain: its generic passwords of a service (e.g. `todai.notion`), by account. Read and written
/// through the `security` tool, the secrets themselves never in its arguments, where other processes could see them.
///
/// Through `security` rather than the Security framework: todai isn't signed with a stable identity, so an item it
/// created itself could only be read again by the same build without asking for the user's password. The items
/// `security` creates are readable by any build alike, as by any process of the user's through `security`: as are the
/// GitHub CLI's tokens.
pub struct Keychain {
    service: &'static str,
    security: PathBuf,
}

impl Keychain {
    pub fn new(service: &'static str) -> Self {
        Self {
            service,
            security: PathBuf::from(SECURITY),
        }
    }

    async fn run(
        &self,
        args: &[&str],
        input: Option<&str>,
    ) -> std::result::Result<Vec<u8>, RunError> {
        let mut command = Command::new(&self.security);
        command.args(args).stdin(Stdio::null());
        process::run_with_input(command, input.map(str::as_bytes), TIMEOUT).await
    }
}

fn keychain_error(action: &str, error: RunError) -> TodaiError {
    TodaiError::Keychain(match error {
        RunError::NotFound => format!("could not {action}: {SECURITY} was not found"),
        RunError::TimedOut => {
            format!("could not {action}: {SECURITY} took longer than {TIMEOUT:?}")
        }
        RunError::Failed { said, .. } => format!("could not {action}: {said}"),
        RunError::Io(error) => format!("could not {action}: {error}"),
    })
}

/// Fails unless `value` fits on one line of `security`'s commands.
fn ensure_one_line(what: &str, value: &str) -> Result<()> {
    if value.is_empty() || value.chars().any(char::is_control) {
        return Err(TodaiError::Keychain(format!(
            "the {what} must be a single line, not empty"
        )));
    }
    Ok(())
}

/// `value` as a quoted argument of `security`'s interactive commands, which may hold any character but line breaks.
fn quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

impl SecretStore for Keychain {
    async fn get(&self, account: &str) -> Result<Option<String>> {
        ensure_one_line("account", account)?;
        let printed = match self
            .run(
                &[
                    "find-generic-password",
                    "-s",
                    self.service,
                    "-a",
                    account,
                    "-w",
                ],
                None,
            )
            .await
        {
            Ok(printed) => printed,
            Err(RunError::Failed {
                code: Some(ITEM_NOT_FOUND),
                ..
            }) => return Ok(None),
            Err(error) => return Err(keychain_error("read a secret", error)),
        };
        let secret = String::from_utf8(printed)
            .map_err(|_| TodaiError::Keychain("a secret isn't text".into()))?;
        // Printed with a line break
        Ok(Some(secret.strip_suffix('\n').unwrap_or(&secret).into()))
    }

    async fn set(&self, account: &str, secret: &str) -> Result<()> {
        ensure_one_line("account", account)?;
        ensure_one_line("secret", secret)?;
        // On its standard input, read as commands: `-U` updates the item if there's one already
        let command = format!(
            "add-generic-password -U -s {} -a {} -w {}\n",
            quote(self.service),
            quote(account),
            quote(secret)
        );
        self.run(&["-i"], Some(&command))
            .await
            .map_err(|error| keychain_error("keep a secret", error))?;
        // `security -i` exits with the error of a command that failed, if any: checked again, as what matters
        if self.get(account).await?.as_deref() != Some(secret) {
            return Err(TodaiError::Keychain("the secret wasn't kept".into()));
        }
        Ok(())
    }

    async fn delete(&self, account: &str) -> Result<bool> {
        ensure_one_line("account", account)?;
        match self
            .run(
                &["delete-generic-password", "-s", self.service, "-a", account],
                None,
            )
            .await
        {
            Ok(_) => Ok(true),
            Err(RunError::Failed {
                code: Some(ITEM_NOT_FOUND),
                ..
            }) => Ok(false),
            Err(error) => Err(keychain_error("delete a secret", error)),
        }
    }
}

/// An in-memory keychain, for tests.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct FakeSecrets {
    secrets: std::sync::Mutex<std::collections::HashMap<String, String>>,
    reads: std::sync::atomic::AtomicUsize,
}

#[cfg(test)]
impl FakeSecrets {
    pub fn with(account: &str, secret: &str) -> Self {
        let secrets = Self::default();
        secrets.lock().insert(account.into(), secret.into());
        secrets
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, std::collections::HashMap<String, String>> {
        self.secrets.lock().unwrap()
    }

    /// The secret of `account`, without counting it as read.
    pub fn peek(&self, account: &str) -> Option<String> {
        self.lock().get(account).cloned()
    }

    /// How many times a secret was read.
    pub fn reads(&self) -> usize {
        self.reads.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
impl SecretStore for FakeSecrets {
    async fn get(&self, account: &str) -> Result<Option<String>> {
        self.reads.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(self.peek(account))
    }

    async fn set(&self, account: &str, secret: &str) -> Result<()> {
        self.lock().insert(account.into(), secret.into());
        Ok(())
    }

    async fn delete(&self, account: &str) -> Result<bool> {
        Ok(self.lock().remove(account).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_what_security_reads_as_one_argument() {
        assert_eq!(quote("ntn_abc123"), r#""ntn_abc123""#);
        assert_eq!(quote(r#"a b"c\d"#), r#""a b\"c\\d""#);
    }

    #[test]
    fn only_single_lines_fit_security_commands() {
        assert!(ensure_one_line("secret", "ntn_abc").is_ok());
        assert!(ensure_one_line("secret", "").is_err());
        assert!(ensure_one_line("secret", "ntn_abc\ndelete-keychain").is_err());
        assert!(ensure_one_line("secret", "ntn\0abc").is_err());
    }

    /// Writes to, reads from and deletes from the user's login keychain, under a service of its own: run with
    /// `cargo test -- --ignored keychain`.
    #[tokio::test]
    #[ignore = "uses the user's login keychain"]
    async fn keychain_keeps_reads_and_deletes_secrets() {
        let keychain = Keychain::new("todai.test");
        let account = format!("test-{}", std::process::id());
        let tricky = r#"ntn_a b"c\d"#;

        assert_eq!(keychain.get(&account).await.unwrap(), None);
        keychain.set(&account, tricky).await.unwrap();
        assert_eq!(
            keychain.get(&account).await.unwrap().as_deref(),
            Some(tricky)
        );
        keychain.set(&account, "ntn_replaced").await.unwrap();
        assert_eq!(
            keychain.get(&account).await.unwrap().as_deref(),
            Some("ntn_replaced")
        );
        assert!(keychain.delete(&account).await.unwrap());
        assert!(!keychain.delete(&account).await.unwrap());
        assert_eq!(keychain.get(&account).await.unwrap(), None);
    }
}
