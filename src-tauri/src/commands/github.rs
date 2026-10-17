use serde::Serialize;
use tauri::State;

use crate::{
    commands::workspaces::workspace,
    errors::{Result, TodaiError},
    gh::GhFinder,
    github::{query::RateLimit, GitHub},
    integrations::{IntegrationProblem, Provider},
    models::{
        app::Workspace,
        integrations::{GhAccount, GhStatus},
    },
    state::AppState,
    workspaces::{self, LinksBehaviors},
};

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

/// A workspace's GitHub connection, as GitHub sees it.
#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GitHubApiStatus {
    /// The connected account's rate limit, if GitHub could be asked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimit>,
    /// Why GitHub couldn't be asked, if so.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<IntegrationProblem>,
}

/// Asks GitHub for the rate limit of the account the workspace `workspace_id` is connected to: which also checks that
/// GitHub accepts its token. Fails if it isn't connected.
#[tauri::command]
pub async fn get_github_api_status(
    state: State<'_, AppState>,
    finder: State<'_, GhFinder>,
    github: State<'_, GitHub>,
    workspace_id: String,
) -> Result<GitHubApiStatus> {
    let login = workspaces::get(&state.app_db(), &workspace_id)?
        .github_login
        .ok_or_else(|| {
            TodaiError::CommandError(format!(
                "Workspace {workspace_id} isn't connected to GitHub"
            ))
        })?;
    let result = github.rate_limit(&*finder, &login).await;
    state.note(Provider::GitHub, &login, &result);
    Ok(match result {
        Ok(rate_limit) => GitHubApiStatus {
            rate_limit: Some(rate_limit),
            problem: None,
        },
        Err(problem) => GitHubApiStatus {
            rate_limit: None,
            problem: Some(problem),
        },
    })
}

/// Sets how the links of the workspace `workspace_id` behave: see `Workspace`.
#[tauri::command]
pub async fn set_workspace_links_behaviors(
    state: State<'_, AppState>,
    workspace_id: String,
    links_auto_refresh: bool,
    links_suggest_done: bool,
    links_paste_to_create: bool,
) -> Result<Workspace> {
    let app_db = state.app_db();
    let behaviors = LinksBehaviors {
        auto_refresh: links_auto_refresh,
        suggest_done: links_suggest_done,
        paste_to_create: links_paste_to_create,
    };
    let entry = workspaces::set_links_behaviors(&app_db, &workspace_id, behaviors)?;
    log::info!(
        "Set the links behaviors of workspace {:?}: {behaviors:?}",
        entry.name
    );
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
}
