use serde::Serialize;
use tauri::State;

use jiff::Timestamp;

use crate::{
    commands::workspaces::workspace,
    errors::{Result, TodaiError},
    integrations::{IntegrationProblem, Provider},
    models::{app::Workspace, integrations::NotionAccount},
    notion::{
        rules::{self, NotionDoneRule},
        snapshot::SchemaColumn,
        Notion,
    },
    secrets::{Keychain, SecretStore},
    state::AppState,
    workspaces,
};

/// Longest token accepted, in bytes: Notion's are far shorter.
const MAX_TOKEN_LENGTH: usize = 1000;

/// What connecting a workspace to Notion gave.
#[derive(Serialize, Debug)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum NotionConnectOutcome {
    /// Notion accepted the token, now kept in the Keychain.
    Connected { workspace: Box<Workspace> },
    /// Notion couldn't vouch for the token (e.g. it refused it, or couldn't be reached): nothing was kept, and the
    /// workspace's connection is as it was.
    NotConnected { problem: IntegrationProblem },
}

/// Connects the workspace `workspace_id` to Notion with the personal access token `token`, instead of any other one:
/// once Notion tells whose it is, it's kept in the Keychain. Fails if it can't be kept.
#[tauri::command]
pub async fn connect_notion(
    state: State<'_, AppState>,
    notion: State<'_, Notion>,
    keychain: State<'_, Keychain>,
    workspace_id: String,
    token: String,
) -> Result<NotionConnectOutcome> {
    connect(&state, &notion, &*keychain, &workspace_id, &token).await
}

/// Disconnects the workspace `workspace_id` from Notion, deleting its token from the Keychain: its links then only show
/// what their URL tells.
#[tauri::command]
pub async fn disconnect_notion(
    state: State<'_, AppState>,
    notion: State<'_, Notion>,
    keychain: State<'_, Keychain>,
    workspace_id: String,
) -> Result<Workspace> {
    disconnect(&state, &notion, &*keychain, &workspace_id).await
}

/// A workspace's Notion connection, as Notion sees it.
#[derive(Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotionApiStatus {
    /// Whose token it is, if Notion could be asked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<NotionAccount>,
    /// Why Notion couldn't be asked, if so: e.g. it refused the token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<IntegrationProblem>,
}

/// Asks Notion whose token the workspace `workspace_id` uses: which also checks that Notion still accepts it. Fails if
/// it isn't connected to Notion.
#[tauri::command]
pub async fn get_notion_api_status(
    state: State<'_, AppState>,
    notion: State<'_, Notion>,
    keychain: State<'_, Keychain>,
    workspace_id: String,
) -> Result<NotionApiStatus> {
    api_status(&state, &notion, &*keychain, &workspace_id).await
}

/// The done rules of the active workspace's Notion databases, by their title.
#[tauri::command]
pub async fn list_notion_done_rules(state: State<'_, AppState>) -> Result<Vec<NotionDoneRule>> {
    rules::list(&*state.db()?)
}

/// Sets the done rule of a Notion database of the active workspace, instead of any other one: its pages are then done
/// once its column holds one of its options meaning done (or is checked), rather than once their Status is in its
/// Complete group.
#[tauri::command]
pub async fn set_notion_done_rule(
    state: State<'_, AppState>,
    rule: NotionDoneRule,
) -> Result<NotionDoneRule> {
    let saved = rules::set(&*state.db()?, &rule, Timestamp::now().as_second())?;
    log::info!(
        "Set the done rule of the Notion database {}: column {}",
        saved.data_source_id,
        saved.property_id
    );
    Ok(saved)
}

/// Deletes the done rule of the Notion database `data_source_id`, returning whether there was one: its pages are then
/// done once their Status is in its Complete group again.
#[tauri::command]
pub async fn delete_notion_done_rule(
    state: State<'_, AppState>,
    data_source_id: String,
) -> Result<bool> {
    rules::delete(&*state.db()?, &data_source_id)
}

/// A Notion database's columns that can tell its pages are done, to pick from for its done rule.
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NotionColumns {
    pub title: String,
    pub columns: Vec<SchemaColumn>,
}

/// Asks Notion for the columns of the database `data_source_id` that can tell its pages are done, through the active
/// workspace's connection. Fails if it isn't connected, Notion couldn't be asked, or its token can't read it.
#[tauri::command]
pub async fn get_notion_columns(
    state: State<'_, AppState>,
    notion: State<'_, Notion>,
    keychain: State<'_, Keychain>,
    data_source_id: String,
) -> Result<NotionColumns> {
    columns(&state, &notion, &*keychain, &data_source_id).await
}

async fn columns(
    state: &AppState,
    notion: &Notion,
    secrets: &impl SecretStore,
    data_source_id: &str,
) -> Result<NotionColumns> {
    let workspace_id = {
        let app_db = state.app_db();
        let id = workspaces::active_id(&app_db)?.ok_or(TodaiError::NoActiveWorkspace)?;
        let entry = workspaces::get(&app_db, &id)?;
        if entry.notion_user_id.is_none() {
            return Err(TodaiError::CommandError(
                "This workspace isn't connected to Notion".into(),
            ));
        }
        entry.id
    };
    let result = notion
        .fresh_schema(secrets, &workspace_id, data_source_id)
        .await;
    state.note(Provider::Notion, &workspace_id, &result);
    match result {
        Ok(Some(schema)) => Ok(NotionColumns {
            title: schema.title,
            columns: schema.columns,
        }),
        Ok(None) => Err(TodaiError::CommandError(
            "This workspace's Notion token can't read this database".into(),
        )),
        Err(problem) => Err(TodaiError::CommandError(format!(
            "Could not ask Notion for the database's columns: {problem}"
        ))),
    }
}

/// `token`, trimmed, if it may be a token at all.
fn valid_token(token: &str) -> Result<&str> {
    let token = token.trim();
    if token.is_empty() {
        return Err(TodaiError::CommandError("The Notion token is empty".into()));
    }
    if token.len() > MAX_TOKEN_LENGTH || token.chars().any(char::is_whitespace) {
        return Err(TodaiError::CommandError(
            "This isn't a Notion token: copy it again from Notion's Developer portal".into(),
        ));
    }
    Ok(token)
}

async fn connect(
    state: &AppState,
    notion: &Notion,
    secrets: &impl SecretStore,
    workspace_id: &str,
    token: &str,
) -> Result<NotionConnectOutcome> {
    let token = valid_token(token)?;
    // An unknown workspace fails before Notion is asked
    workspaces::get(&state.app_db(), workspace_id)?;
    let account = match notion.account_of(token).await {
        Ok(account) => account,
        Err(problem) => {
            log::warn!(
                "Notion didn't vouch for the token given for workspace {workspace_id}: {problem}"
            );
            return Ok(NotionConnectOutcome::NotConnected { problem });
        }
    };

    // Kept, then recorded: should recording fail, the token it replaced (if any) is put back
    let previous = secrets.get(workspace_id).await?;
    secrets.set(workspace_id, token).await?;
    let recorded = workspaces::set_notion_account(&state.app_db(), workspace_id, Some(&account));
    let entry = match recorded {
        Ok(entry) => entry,
        Err(error) => {
            let restored = match &previous {
                Some(previous) => secrets.set(workspace_id, previous).await,
                None => secrets.delete(workspace_id).await.map(|_| ()),
            };
            if let Err(restoring) = restored {
                log::error!(
                    "Could not restore the Notion token of workspace {workspace_id}: {restoring}"
                );
            }
            return Err(error);
        }
    };
    notion.remember_token(workspace_id, token);
    state.note::<()>(Provider::Notion, workspace_id, &Ok(()));
    log::info!(
        "Connected workspace {:?} to Notion as user {}",
        entry.name,
        account.user_id
    );
    let app_db = state.app_db();
    Ok(NotionConnectOutcome::Connected {
        workspace: Box::new(workspace(state, &app_db, entry)?),
    })
}

async fn disconnect(
    state: &AppState,
    notion: &Notion,
    secrets: &impl SecretStore,
    workspace_id: &str,
) -> Result<Workspace> {
    workspaces::get(&state.app_db(), workspace_id)?;
    // The token first: should deleting it fail, the workspace stays connected, rather than its token left behind
    secrets.delete(workspace_id).await?;
    forget(state, notion, workspace_id);
    let app_db = state.app_db();
    let entry = workspaces::set_notion_account(&app_db, workspace_id, None)?;
    log::info!("Disconnected workspace {:?} from Notion", entry.name);
    workspace(state, &app_db, entry)
}

/// Forgets what's known in memory of the Notion connection of the workspace `workspace_id`.
fn forget(state: &AppState, notion: &Notion, workspace_id: &str) {
    notion.forget(workspace_id);
    state.note::<()>(Provider::Notion, workspace_id, &Ok(()));
}

/// Deletes the Notion token of the workspace `workspace_id`, once it's removed, if it was connected to Notion. Only
/// logged if it fails: the workspace is gone anyway.
pub(crate) async fn forget_removed(
    state: &AppState,
    notion: &Notion,
    secrets: &impl SecretStore,
    workspace_id: &str,
) {
    forget(state, notion, workspace_id);
    match secrets.delete(workspace_id).await {
        Ok(true) => log::info!("Deleted the Notion token of the removed workspace {workspace_id}"),
        Ok(false) => {}
        Err(error) => log::error!(
            "Could not delete the Notion token of the removed workspace {workspace_id}: {error}"
        ),
    }
}

async fn api_status(
    state: &AppState,
    notion: &Notion,
    secrets: &impl SecretStore,
    workspace_id: &str,
) -> Result<NotionApiStatus> {
    if workspaces::get(&state.app_db(), workspace_id)?
        .notion_user_id
        .is_none()
    {
        return Err(TodaiError::CommandError(format!(
            "Workspace {workspace_id} isn't connected to Notion"
        )));
    }
    let result = notion.account(secrets, workspace_id).await;
    state.note(Provider::Notion, workspace_id, &result);
    Ok(match result {
        Ok(account) => NotionApiStatus {
            account: Some(account),
            problem: None,
        },
        Err(problem) => NotionApiStatus {
            account: None,
            problem: Some(problem),
        },
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::{
        database::{self, models::DbWorkspace},
        secrets::FakeSecrets,
        testing::{ok, FakeServer, Response},
    };

    const NOW: i64 = 1_791_000_000;

    struct Setup {
        _dir: tempfile::TempDir,
        state: AppState,
        work: DbWorkspace,
    }

    fn setup() -> Setup {
        let dir = tempfile::tempdir().unwrap();
        let app_db = database::open_test_database(&database::APP);
        let (work, _) =
            workspaces::create(&app_db, "Work", &dir.path().join("work.sqlite3"), NOW).unwrap();
        Setup {
            _dir: dir,
            state: AppState::new(app_db, None, PathBuf::from("backups")),
            work,
        }
    }

    fn me() -> Response {
        ok(json!({
            "object": "user", "id": "u1", "name": "Pierre-Yves", "type": "person",
            "person": { "email": "py@example.org" },
        })
        .to_string())
    }

    fn unauthorized() -> Response {
        (
            401,
            Vec::new(),
            r#"{"object":"error","status":401,"code":"unauthorized","message":"API token is invalid."}"#.into(),
        )
    }

    fn stored(s: &Setup) -> DbWorkspace {
        workspaces::get(&s.state.app_db(), &s.work.id).unwrap()
    }

    #[tokio::test]
    async fn connecting_keeps_the_token_notion_vouched_for() {
        let s = setup();
        let server = FakeServer::serve(vec![me()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::default();

        let outcome = connect(&s.state, &notion, &secrets, &s.work.id, "  ntn_good \n")
            .await
            .unwrap();
        let NotionConnectOutcome::Connected { workspace } = outcome else {
            panic!("not connected: {outcome:?}");
        };
        assert_eq!(
            serde_json::to_value(&workspace).unwrap()["notion"],
            json!({ "userId": "u1", "name": "Pierre-Yves", "email": "py@example.org" })
        );
        assert_eq!(secrets.peek(&s.work.id).as_deref(), Some("ntn_good"));
        assert_eq!(stored(&s).notion_user_id.as_deref(), Some("u1"));
        assert_eq!(server.authorizations(), ["Bearer ntn_good"]);
    }

    #[tokio::test]
    async fn connecting_keeps_nothing_notion_refused() {
        let s = setup();
        let server = FakeServer::serve(vec![unauthorized()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::default();

        let outcome = connect(&s.state, &notion, &secrets, &s.work.id, "ntn_bad")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            NotionConnectOutcome::NotConnected {
                problem: IntegrationProblem::TokenRefused
            }
        ));
        assert_eq!(
            serde_json::to_value(&outcome).unwrap(),
            json!({ "kind": "not_connected", "problem": { "kind": "token_refused" } })
        );
        assert_eq!(secrets.peek(&s.work.id), None);
        assert_eq!(stored(&s).notion_user_id, None);
    }

    #[tokio::test]
    async fn connecting_while_offline_keeps_the_previous_token() {
        let s = setup();
        let notion = Notion::with_endpoint(&FakeServer::unreachable_url()).unwrap();
        let secrets = FakeSecrets::with(&s.work.id, "ntn_previous");

        let outcome = connect(&s.state, &notion, &secrets, &s.work.id, "ntn_new")
            .await
            .unwrap();
        assert!(matches!(
            outcome,
            NotionConnectOutcome::NotConnected {
                problem: IntegrationProblem::Offline
            }
        ));
        assert_eq!(secrets.peek(&s.work.id).as_deref(), Some("ntn_previous"));
    }

    #[tokio::test]
    async fn connecting_rejects_what_isnt_a_token_without_asking_notion() {
        let s = setup();
        let server = FakeServer::serve(Vec::new());
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::default();

        for token in ["", "   ", "ntn_a ntn_b", &"a".repeat(MAX_TOKEN_LENGTH + 1)] {
            assert!(
                connect(&s.state, &notion, &secrets, &s.work.id, token)
                    .await
                    .is_err(),
                "{token:?}"
            );
        }
        assert!(connect(&s.state, &notion, &secrets, "unknown", "ntn_t")
            .await
            .is_err());
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn connecting_again_clears_the_attention_and_uses_the_new_token() {
        let s = setup();
        // Refused from memory, then once read again from the Keychain
        let server = FakeServer::serve(vec![me(), unauthorized(), unauthorized(), me(), me()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::default();
        connect(&s.state, &notion, &secrets, &s.work.id, "ntn_first")
            .await
            .unwrap();

        // Expired since
        let status = api_status(&s.state, &notion, &secrets, &s.work.id)
            .await
            .unwrap();
        assert_eq!(status.problem, Some(IntegrationProblem::TokenRefused));
        assert_eq!(
            s.state.attention(Provider::Notion, &s.work.id),
            Some(IntegrationProblem::TokenRefused)
        );

        connect(&s.state, &notion, &secrets, &s.work.id, "ntn_second")
            .await
            .unwrap();
        assert_eq!(s.state.attention(Provider::Notion, &s.work.id), None);
        let status = api_status(&s.state, &notion, &secrets, &s.work.id)
            .await
            .unwrap();
        assert_eq!(
            status.account.map(|account| account.user_id).as_deref(),
            Some("u1")
        );
        assert_eq!(
            server.authorizations(),
            [
                "Bearer ntn_first",
                "Bearer ntn_first",
                "Bearer ntn_first",
                "Bearer ntn_second",
                "Bearer ntn_second"
            ]
        );
    }

    #[tokio::test]
    async fn disconnecting_deletes_the_token() {
        let s = setup();
        let server = FakeServer::serve(vec![me()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::default();
        connect(&s.state, &notion, &secrets, &s.work.id, "ntn_t")
            .await
            .unwrap();

        let workspace = disconnect(&s.state, &notion, &secrets, &s.work.id)
            .await
            .unwrap();
        assert!(workspace.notion.is_none());
        assert_eq!(secrets.peek(&s.work.id), None);
        assert_eq!(stored(&s).notion_user_id, None);
        assert!(api_status(&s.state, &notion, &secrets, &s.work.id)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn a_removed_workspace_s_token_is_deleted() {
        let s = setup();
        let notion = Notion::with_endpoint(&FakeServer::unreachable_url()).unwrap();
        let secrets = FakeSecrets::with(&s.work.id, "ntn_t");
        notion.remember_token(&s.work.id, "ntn_t");

        forget_removed(&s.state, &notion, &secrets, &s.work.id).await;
        assert_eq!(secrets.peek(&s.work.id), None);
        // Not remembered either
        assert_eq!(
            notion.account(&secrets, &s.work.id).await,
            Err(IntegrationProblem::TokenMissing)
        );
    }

    #[tokio::test]
    async fn the_status_tells_when_the_token_is_missing_from_the_keychain() {
        let s = setup();
        let server = FakeServer::serve(vec![me()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::default();
        connect(&s.state, &notion, &secrets, &s.work.id, "ntn_t")
            .await
            .unwrap();
        // Deleted from Keychain Access, then todai restarted
        secrets.delete(&s.work.id).await.unwrap();
        notion.forget(&s.work.id);

        let status = api_status(&s.state, &notion, &secrets, &s.work.id)
            .await
            .unwrap();
        assert_eq!(status.problem, Some(IntegrationProblem::TokenMissing));
        let entry = stored(&s);
        let app_db = s.state.app_db();
        let workspace = workspace(&s.state, &app_db, entry).unwrap();
        assert_eq!(
            workspace.notion_attention,
            Some(IntegrationProblem::TokenMissing)
        );
    }

    #[tokio::test]
    async fn the_columns_of_a_database_come_fresh_from_notion() {
        use crate::notion::snapshot::{tests::tickets_schema_json, ColumnKind};
        let s = setup();
        workspaces::set_active(&s.state.app_db(), &s.work.id, NOW).unwrap();
        let server = FakeServer::serve(vec![
            me(),
            ok(tickets_schema_json().to_string()),
            ok(tickets_schema_json().to_string()),
        ]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::default();
        assert!(columns(&s.state, &notion, &secrets, "ds1").await.is_err());
        connect(&s.state, &notion, &secrets, &s.work.id, "ntn_t")
            .await
            .unwrap();

        let found = columns(&s.state, &notion, &secrets, "ds1").await.unwrap();
        assert_eq!(found.title, "Tickets");
        let stage = found.columns.iter().find(|c| c.name == "Stage").unwrap();
        assert_eq!(stage.kind, ColumnKind::Select);
        assert_eq!(stage.options.len(), 3);
        // Asked again, rather than kept
        columns(&s.state, &notion, &secrets, "ds1").await.unwrap();
        assert_eq!(server.requests().len(), 3);
    }
}
