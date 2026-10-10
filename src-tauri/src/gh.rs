//! The GitHub CLI (`gh`), through which todai uses GitHub: rather than storing tokens itself, it asks `gh` for the one
//! of the account a workspace is connected to.

use std::{
    collections::HashMap,
    env, io,
    path::{Path, PathBuf},
    process::Stdio,
    sync::{Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::{process::Command, time::timeout};

use crate::errors::{Result, TodaiError};

/// The only GitHub host supported: links are only recognized on github.com.
pub const HOST: &str = "github.com";

/// How long a `gh` command may take: `gh auth status` checks each account online.
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);

/// How long looking `gh` up through a login shell may take: the user's startup files can be slow.
const SHELL_LOOKUP_TIMEOUT: Duration = Duration::from_secs(3);

/// Variables that would make `gh` use another token than the chosen account's (e.g. exported in the terminal `tauri
/// dev` was started from), or another host.
const OVERRIDING_VARS: [&str; 5] = [
    "GH_TOKEN",
    "GITHUB_TOKEN",
    "GH_ENTERPRISE_TOKEN",
    "GITHUB_ENTERPRISE_TOKEN",
    "GH_HOST",
];

const EXECUTABLE: &str = if cfg!(windows) { "gh.exe" } else { "gh" };

/// An account `gh` knows on github.com.
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GhAccount {
    pub login: String,
    /// The one `gh` uses by default (`gh auth switch` changes it).
    pub active: bool,
    /// Whether its token works: not once revoked or expired, until signing in again with `gh auth login`.
    pub signed_in: bool,
}

/// What `gh auth status --json hosts` prints: the accounts of each host.
#[derive(Deserialize)]
struct AuthStatus {
    hosts: HashMap<String, Vec<AuthStatusAccount>>,
}

#[derive(Deserialize)]
struct AuthStatusAccount {
    login: String,
    active: bool,
    /// `success` when its token works; otherwise e.g. `error` or `timeout`.
    state: String,
}

/// The accounts on github.com in what `gh auth status --json hosts` printed.
fn parse_accounts(json: &[u8]) -> Result<Vec<GhAccount>> {
    let mut status: AuthStatus = serde_json::from_slice(json)?;
    Ok(status
        .hosts
        .remove(HOST)
        .unwrap_or_default()
        .into_iter()
        .map(|account| GhAccount {
            login: account.login,
            active: account.active,
            signed_in: account.state == "success",
        })
        .collect())
}

/// The `gh` executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GhCli {
    path: PathBuf,
    timeout: Duration,
}

impl GhCli {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            timeout: COMMAND_TIMEOUT,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The accounts `gh` knows on github.com, signed in or not anymore: none if it was never signed in.
    pub async fn accounts(&self) -> Result<Vec<GhAccount>> {
        let output = self
            .run(&["auth", "status", "--json", "hosts", "--hostname", HOST])
            .await?;
        parse_accounts(&output)
    }

    /// The token of the account `login` on github.com. To keep in memory only, and never log.
    pub async fn token(&self, login: &str) -> Result<String> {
        let output = self
            .run(&["auth", "token", "--hostname", HOST, "--user", login])
            .await?;
        let token = String::from_utf8(output).map_err(|_| {
            TodaiError::GhCli("`gh auth token` printed something else than a token".into())
        })?;
        match token.trim() {
            "" => Err(TodaiError::GhCli(format!(
                "`gh auth token` printed no token for {login}"
            ))),
            token => Ok(token.into()),
        }
    }

    /// `gh` with `args`, which never waits for an answer, and only uses the accounts it's signed in to.
    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(&self.path);
        command
            .args(args)
            .env("GH_PROMPT_DISABLED", "1")
            .env("GH_NO_UPDATE_NOTIFIER", "1")
            .env("NO_COLOR", "1")
            .stdin(Stdio::null())
            .kill_on_drop(true);
        for var in OVERRIDING_VARS {
            command.env_remove(var);
        }
        command
    }

    /// Runs `gh` with `args`, returning what it printed. Fails if it exits with an error (with what it said), or takes
    /// too long: it's then killed.
    async fn run(&self, args: &[&str]) -> Result<Vec<u8>> {
        // E.g. `gh auth status`: the rest may be an account's login.
        let name = format!("gh {}", args[..args.len().min(2)].join(" "));
        let output = match timeout(self.timeout, self.command(args).output()).await {
            Err(_) => {
                return Err(TodaiError::GhCli(format!(
                    "`{name}` took longer than {:?}",
                    self.timeout
                )))
            }
            Ok(Err(error)) if error.kind() == io::ErrorKind::NotFound => {
                return Err(TodaiError::GhNotFound)
            }
            Ok(output) => output?,
        };
        if !output.status.success() {
            let said = String::from_utf8_lossy(&output.stderr);
            return Err(TodaiError::GhCli(format!(
                "`{name}` failed: {}",
                said.trim()
            )));
        }
        Ok(output.stdout)
    }
}

/// Where to look for `gh`, in order.
struct Lookup {
    dirs: Vec<PathBuf>,
    /// Last, asked `command -v gh` as a login shell: it reads the user's profile, where `PATH` usually gets extended.
    shell: Option<PathBuf>,
    shell_timeout: Duration,
}

impl Lookup {
    /// Where `gh` is usually installed first: apps launched from the Dock or the Finder only get
    /// `/usr/bin:/bin:/usr/sbin:/sbin` as their `PATH`. Then the `PATH`, as when started from a terminal.
    fn from_env() -> Self {
        let mut dirs: Vec<PathBuf> = [
            "/opt/homebrew/bin",
            "/usr/local/bin",
            "/opt/local/bin",
            "/run/current-system/sw/bin",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect();
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            dirs.extend([home.join(".nix-profile/bin"), home.join(".local/bin")]);
        }
        if let Some(path) = env::var_os("PATH") {
            dirs.extend(env::split_paths(&path));
        }
        let shell = cfg!(unix).then(|| {
            env::var_os("SHELL")
                .map(PathBuf::from)
                .filter(|shell| shell.is_absolute())
                .unwrap_or_else(|| PathBuf::from("/bin/zsh"))
        });
        Self {
            dirs,
            shell,
            shell_timeout: SHELL_LOOKUP_TIMEOUT,
        }
    }

    async fn find(&self) -> Option<PathBuf> {
        if let Some(path) = self
            .dirs
            .iter()
            .map(|dir| dir.join(EXECUTABLE))
            .find(|path| is_executable(path))
        {
            return Some(path);
        }

        let mut command = Command::new(self.shell.as_ref()?);
        command
            .args(["-l", "-c", "command -v gh"])
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        let output = timeout(self.shell_timeout, command.output())
            .await
            .ok()?
            .ok()?;
        // The user's startup files may print lines of their own first.
        let stdout = String::from_utf8(output.stdout).ok()?;
        let path = PathBuf::from(stdout.lines().last()?.trim());
        (output.status.success() && path.is_absolute() && is_executable(&path)).then_some(path)
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

/// What looking for `gh` gave, last time.
#[derive(Clone, Default)]
enum GhLookup {
    #[default]
    NotYet,
    /// Remembered too: looking again in vain is the slowest, up to [`SHELL_LOOKUP_TIMEOUT`].
    Missing,
    Found(GhCli),
}

/// Finds `gh` once, then remembers where: until asked to look again, e.g. after installing it.
#[derive(Default)]
pub struct GhFinder {
    lookup: Mutex<GhLookup>,
}

impl GhFinder {
    /// `gh`, if installed: looked for again if `refind`.
    pub async fn get(&self, refind: bool) -> Option<GhCli> {
        let remembered = if refind {
            GhLookup::NotYet
        } else {
            self.lock().clone()
        };
        let found = match remembered {
            GhLookup::Found(gh) => return Some(gh),
            GhLookup::Missing => return None,
            GhLookup::NotYet => Lookup::from_env().find().await.map(GhCli::new),
        };
        match &found {
            Some(gh) => log::info!("Found the GitHub CLI at {}", gh.path.display()),
            None => log::warn!("The GitHub CLI (gh) was not found"),
        }
        *self.lock() = found.clone().map_or(GhLookup::Missing, GhLookup::Found);
        found
    }

    fn lock(&self) -> MutexGuard<'_, GhLookup> {
        self.lookup.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::{collections::HashMap, ffi::OsStr, fs, os::unix::fs::PermissionsExt};

    use super::*;

    /// Writes the executable shell script `name` in `dir`, running `body`.
    fn script(dir: &Path, name: &str, body: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    /// A fake `gh` in `dir`, which runs `body` if called with `expected_args`, and fails otherwise.
    fn fake_gh(dir: &Path, expected_args: &str, body: &str) -> GhCli {
        GhCli::new(script(
            dir,
            "gh",
            &format!(
                "[ \"$*\" = \"{expected_args}\" ] || {{ echo \"unexpected arguments: $*\" >&2; exit 2; }}\n{body}"
            ),
        ))
    }

    fn lookup(dirs: Vec<PathBuf>, shell: Option<PathBuf>) -> Lookup {
        Lookup {
            dirs,
            shell,
            shell_timeout: Duration::from_secs(5),
        }
    }

    const AUTH_STATUS_ARGS: &str = "auth status --json hosts --hostname github.com";

    /// As printed by gh 2.102, with another host to ignore.
    const AUTH_STATUS: &str = r#"{"hosts":{
        "github.com":[
            {"active":true,"gitProtocol":"ssh","host":"github.com","login":"pierreyoda","scopes":"gist, read:org, repo","state":"success","tokenSource":"keyring"},
            {"active":false,"error":"The token in keyring is invalid.","host":"github.com","login":"acme-dev","state":"error","tokenSource":"keyring"}
        ],
        "ghe.example.com":[
            {"active":true,"host":"ghe.example.com","login":"someone","state":"success","tokenSource":"keyring"}
        ]
    }}"#;

    #[tokio::test]
    async fn finds_gh_in_the_first_directory_having_it() {
        let dir = tempfile::tempdir().unwrap();
        let [not_executable, first, second] =
            ["not-executable", "first", "second"].map(|name| dir.path().join(name));
        for subdir in [&not_executable, &first, &second] {
            fs::create_dir(subdir).unwrap();
        }
        fs::write(not_executable.join("gh"), "").unwrap();
        let expected = script(&first, "gh", "exit 0");
        script(&second, "gh", "exit 0");

        let dirs = vec![dir.path().join("missing"), not_executable, first, second];
        assert_eq!(lookup(dirs, None).find().await, Some(expected));
    }

    #[tokio::test]
    async fn falls_back_to_asking_a_login_shell() {
        let dir = tempfile::tempdir().unwrap();
        let gh = script(dir.path(), "gh", "exit 0");
        // Like a shell whose startup files greet the user first
        let shell = script(
            dir.path(),
            "shell",
            &format!("echo 'Welcome!'\necho '{}'", gh.display()),
        );
        assert_eq!(lookup(vec![], Some(shell)).find().await, Some(gh));
    }

    #[tokio::test]
    async fn the_login_shell_must_answer_an_executable_in_time() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("gh");
        for body in [
            "echo gh".to_string(),
            format!("echo '{}'", missing.display()),
            format!("echo '{}'; exit 1", missing.display()),
        ] {
            let shell = script(dir.path(), "shell", &body);
            assert_eq!(lookup(vec![], Some(shell)).find().await, None, "{body}");
        }

        let slow = script(dir.path(), "slow", "exec sleep 5");
        let mut slow_lookup = lookup(vec![], Some(slow));
        slow_lookup.shell_timeout = Duration::from_millis(100);
        assert_eq!(slow_lookup.find().await, None);

        assert_eq!(lookup(vec![], None).find().await, None);
    }

    #[test]
    fn parse_accounts_keeps_github_com_with_whether_they_are_signed_in() {
        assert_eq!(
            parse_accounts(AUTH_STATUS.as_bytes()).unwrap(),
            [
                GhAccount {
                    login: "pierreyoda".into(),
                    active: true,
                    signed_in: true,
                },
                GhAccount {
                    login: "acme-dev".into(),
                    active: false,
                    signed_in: false,
                },
            ]
        );
        assert!(parse_accounts(br#"{"hosts":{}}"#).unwrap().is_empty());
        assert!(parse_accounts(b"You are not logged into any GitHub hosts.").is_err());
    }

    #[tokio::test]
    async fn accounts_come_from_gh_auth_status() {
        let dir = tempfile::tempdir().unwrap();
        let gh = fake_gh(
            dir.path(),
            AUTH_STATUS_ARGS,
            &format!("cat <<'EOF'\n{AUTH_STATUS}\nEOF"),
        );
        let accounts = gh.accounts().await.unwrap();
        let logins: Vec<&str> = accounts
            .iter()
            .map(|account| account.login.as_str())
            .collect();
        assert_eq!(logins, ["pierreyoda", "acme-dev"]);
    }

    #[tokio::test]
    async fn accounts_are_none_when_gh_was_never_signed_in() {
        let dir = tempfile::tempdir().unwrap();
        // gh 2.102 says so on stderr, but still prints the JSON and succeeds
        let gh = fake_gh(
            dir.path(),
            AUTH_STATUS_ARGS,
            "echo 'You are not logged into any GitHub hosts.' >&2\necho '{\"hosts\":{}}'",
        );
        assert!(gh.accounts().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn token_is_the_one_of_the_account_without_its_newline() {
        let dir = tempfile::tempdir().unwrap();
        let gh = fake_gh(
            dir.path(),
            "auth token --hostname github.com --user pierreyoda",
            "echo gho_fake",
        );
        assert_eq!(gh.token("pierreyoda").await.unwrap(), "gho_fake");

        let silent = fake_gh(
            dir.path(),
            "auth token --hostname github.com --user pierreyoda",
            "exit 0",
        );
        assert!(matches!(
            silent.token("pierreyoda").await,
            Err(TodaiError::GhCli(_))
        ));
    }

    #[tokio::test]
    async fn failures_carry_what_gh_said() {
        let dir = tempfile::tempdir().unwrap();
        let gh = fake_gh(
            dir.path(),
            "auth token --hostname github.com --user nobody",
            "echo 'no oauth token found for github.com account nobody' >&2\nexit 1",
        );
        match gh.token("nobody").await {
            Err(TodaiError::GhCli(message)) => assert_eq!(
                message,
                "`gh auth token` failed: no oauth token found for github.com account nobody"
            ),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[tokio::test]
    async fn slow_commands_fail_and_missing_executables_are_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let mut slow = fake_gh(dir.path(), AUTH_STATUS_ARGS, "exec sleep 5");
        slow.timeout = Duration::from_millis(100);
        assert!(
            matches!(slow.accounts().await, Err(TodaiError::GhCli(message)) if message.contains("took longer"))
        );

        let missing = GhCli::new(dir.path().join("missing-gh"));
        assert!(matches!(
            missing.accounts().await,
            Err(TodaiError::GhNotFound)
        ));
    }

    #[test]
    fn commands_never_let_the_environment_choose_the_token_nor_wait_for_answers() {
        let command = GhCli::new(PathBuf::from("/opt/homebrew/bin/gh")).command(&["auth", "token"]);
        let envs: HashMap<&OsStr, Option<&OsStr>> = command.as_std().get_envs().collect();
        for var in OVERRIDING_VARS {
            assert_eq!(envs.get(OsStr::new(var)), Some(&None), "{var} is removed");
        }
        assert_eq!(
            envs.get(OsStr::new("GH_PROMPT_DISABLED")),
            Some(&Some(OsStr::new("1")))
        );
    }

    /// Against the real `gh` of the machine, if any: `cargo test real_gh -- --ignored --nocapture`.
    #[tokio::test]
    #[ignore = "depends on the machine's GitHub CLI and its accounts"]
    async fn real_gh_is_found_and_lists_its_accounts() {
        let gh = GhFinder::default()
            .get(false)
            .await
            .expect("gh is installed");
        println!("gh: {}", gh.path().display());
        for account in gh.accounts().await.unwrap() {
            println!("account: {account:?}");
        }
    }
}
