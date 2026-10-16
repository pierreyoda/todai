//! Running other programs, such as the GitHub CLI, without ever waiting on them for too long.

use std::{io, time::Duration};

use tokio::{process::Command, time::timeout};

/// Why a program didn't print what was asked.
#[derive(Debug)]
pub enum RunError {
    /// Its executable doesn't exist.
    NotFound,
    /// It took longer than allowed: it was killed.
    TimedOut,
    /// It exited with an error, saying this on its standard error.
    Failed(String),
    Io(io::Error),
}

/// Runs `command`, returning what it printed on its standard output. Fails if it exits with an error, or takes longer
/// than `limit`: it's then killed.
pub async fn run(mut command: Command, limit: Duration) -> Result<Vec<u8>, RunError> {
    let output = match timeout(limit, command.kill_on_drop(true).output()).await {
        Err(_) => return Err(RunError::TimedOut),
        Ok(Err(error)) if error.kind() == io::ErrorKind::NotFound => {
            return Err(RunError::NotFound)
        }
        Ok(output) => output.map_err(RunError::Io)?,
    };
    if !output.status.success() {
        let said = String::from_utf8_lossy(&output.stderr);
        return Err(RunError::Failed(said.trim().into()));
    }
    Ok(output.stdout)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn sh(script: &str) -> Command {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", script]);
        command
    }

    #[tokio::test]
    async fn returns_what_the_program_printed() {
        let printed = run(sh("echo hello"), Duration::from_secs(5)).await.unwrap();
        assert_eq!(printed, b"hello\n");
    }

    #[tokio::test]
    async fn tells_why_it_failed() {
        assert!(matches!(
            run(sh("echo nope >&2; exit 3"), Duration::from_secs(5)).await,
            Err(RunError::Failed(said)) if said == "nope"
        ));
        assert!(matches!(
            run(Command::new("/nonexistent/program"), Duration::from_secs(5)).await,
            Err(RunError::NotFound)
        ));
        assert!(matches!(
            run(sh("sleep 5"), Duration::from_millis(50)).await,
            Err(RunError::TimedOut)
        ));
    }
}
