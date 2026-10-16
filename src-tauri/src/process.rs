//! Running other programs, such as the GitHub CLI, without ever waiting on them for too long.

use std::{io, process::Stdio, time::Duration};

use tokio::{io::AsyncWriteExt, process::Command, time::timeout};

/// Why a program didn't print what was asked.
#[derive(Debug)]
pub enum RunError {
    /// Its executable doesn't exist.
    NotFound,
    /// It took longer than allowed: it was killed.
    TimedOut,
    /// It exited with an error, with this code (if not killed by a signal), saying this on its standard error.
    Failed {
        code: Option<i32>,
        said: String,
    },
    Io(io::Error),
}

/// Runs `command`, returning what it printed on its standard output. Fails if it exits with an error, or takes longer
/// than `limit`: it's then killed.
pub async fn run(command: Command, limit: Duration) -> Result<Vec<u8>, RunError> {
    run_with_input(command, None, limit).await
}

/// Runs `command` as `run`, writing `input` on its standard input, if any, rather than leaving it empty: e.g. a secret,
/// which no other process can see there, unlike in its arguments.
pub async fn run_with_input(
    mut command: Command,
    input: Option<&[u8]>,
    limit: Duration,
) -> Result<Vec<u8>, RunError> {
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let run = async {
        let mut child = command.spawn()?;
        if let (Some(input), Some(mut stdin)) = (input, child.stdin.take()) {
            stdin.write_all(input).await?;
            // Closed, so that it reads to its end
            drop(stdin);
        }
        child.wait_with_output().await
    };
    let output = match timeout(limit, run).await {
        Err(_) => return Err(RunError::TimedOut),
        Ok(Err(error)) if error.kind() == io::ErrorKind::NotFound => {
            return Err(RunError::NotFound)
        }
        Ok(output) => output.map_err(RunError::Io)?,
    };
    if !output.status.success() {
        return Err(RunError::Failed {
            code: output.status.code(),
            said: String::from_utf8_lossy(&output.stderr).trim().into(),
        });
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
    async fn writes_its_input() {
        let printed = run_with_input(sh("tr a-z A-Z"), Some(b"secret"), Duration::from_secs(5))
            .await
            .unwrap();
        assert_eq!(printed, b"SECRET");
    }

    #[tokio::test]
    async fn tells_why_it_failed() {
        assert!(matches!(
            run(sh("echo nope >&2; exit 44"), Duration::from_secs(5)).await,
            Err(RunError::Failed { code: Some(44), said }) if said == "nope"
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
