use serde::Serialize;
use tauri::State;

use crate::{
    commands::workspaces::{workspace, Workspace},
    errors::{Result, TodaiError},
    gh::{GhAccount, GhFinder},
    state::AppState,
    workspaces,
};

/// What the GitHub CLI offers to connect workspaces with.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GhStatus {
    /// Where `gh` was found, if anywhere.
    pub path: Option<String>,
    /// The accounts it knows on github.com, signed in or not anymore: none without `gh`.
    pub accounts: Vec<GhAccount>,
}

/// Whether the GitHub CLI is installed, with the accounts it knows on github.com. It's looked for once, then
/// remembered: `refind` looks for it again, e.g. after installing it.
#[tauri::command]
pub async fn get_gh_status(finder: State<'_, GhFinder>, refind: Option<bool>) -> Result<GhStatus> {
    let Some(gh) = finder.get(refind.unwrap_or(false)).await else {
        return Ok(GhStatus {
            path: None,
            accounts: Vec::new(),
        });
    };
    let accounts = gh.accounts().await?;
    Ok(GhStatus {
        path: Some(gh.path().display().to_string()),
        accounts,
    })
}

/// Connects the workspace `workspace_id` to GitHub through the GitHub CLI account `login`, instead of any other one.
///
/// Fails unless `gh` is signed in to `login` on github.com.
#[tauri::command]
pub async fn connect_github(
    state: State<'_, AppState>,
    finder: State<'_, GhFinder>,
    workspace_id: String,
    login: String,
) -> Result<Workspace> {
    let gh = finder.get(false).await.ok_or(TodaiError::GhNotFound)?;
    ensure_signed_in(&gh.accounts().await?, &login)?;
    // Locked once `gh` answered, rather than while waiting for it.
    let app_db = state.app_db();
    let entry = workspaces::set_github_login(&app_db, &workspace_id, Some(&login))?;
    log::info!("Connected workspace {:?} to GitHub as {login}", entry.name);
    workspace(&state, &app_db, entry)
}

/// Disconnects the workspace `workspace_id` from GitHub: its links then only show what their URL tells.
#[tauri::command]
pub async fn disconnect_github(
    state: State<'_, AppState>,
    workspace_id: String,
) -> Result<Workspace> {
    let app_db = state.app_db();
    let entry = workspaces::set_github_login(&app_db, &workspace_id, None)?;
    log::info!("Disconnected workspace {:?} from GitHub", entry.name);
    workspace(&state, &app_db, entry)
}

/// Fails unless `login` is among `accounts`, signed in.
fn ensure_signed_in(accounts: &[GhAccount], login: &str) -> Result<()> {
    match accounts.iter().find(|account| account.login == login) {
        Some(account) if account.signed_in => Ok(()),
        Some(_) => Err(TodaiError::CommandError(format!(
            "The GitHub CLI isn't signed in to {login} anymore: sign in again with `gh auth login`"
        ))),
        None => Err(TodaiError::CommandError(format!(
            "The GitHub CLI has no account {login} on github.com: add it with `gh auth login`"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(login: &str, signed_in: bool) -> GhAccount {
        GhAccount {
            login: login.into(),
            active: false,
            signed_in,
        }
    }

    #[test]
    fn only_signed_in_accounts_can_be_connected() {
        let accounts = [account("pierreyoda", true), account("acme-dev", false)];
        assert!(ensure_signed_in(&accounts, "pierreyoda").is_ok());
        assert!(ensure_signed_in(&accounts, "acme-dev").is_err());
        assert!(ensure_signed_in(&accounts, "someone-else").is_err());
        assert!(ensure_signed_in(&[], "pierreyoda").is_err());
    }

    #[test]
    fn gh_status_serializes_in_camel_case() {
        let status = GhStatus {
            path: Some("/opt/homebrew/bin/gh".into()),
            accounts: vec![GhAccount {
                login: "pierreyoda".into(),
                active: true,
                signed_in: true,
            }],
        };
        assert_eq!(
            serde_json::to_value(status).unwrap(),
            serde_json::json!({
                "path": "/opt/homebrew/bin/gh",
                "accounts": [{ "login": "pierreyoda", "active": true, "signedIn": true }],
            })
        );
    }
}
