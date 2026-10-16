//! Notion's REST API, with each workspace's personal access token: read from the Keychain when needed, then kept in
//! memory.

use std::{
    collections::HashMap,
    sync::{Mutex, MutexGuard, PoisonError},
    time::Duration,
};

use jiff::{SignedDuration, Timestamp};
use reqwest::header::{HeaderMap, ACCEPT, RETRY_AFTER};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    errors::{self, TodaiError},
    integrations::IntegrationProblem,
    models::integrations::NotionAccount,
    secrets::SecretStore,
};

pub const ENDPOINT: &str = "https://api.notion.com/v1";

/// The version of the API todai is written for: Notion keeps answering as it did then.
pub const API_VERSION: &str = "2026-03-11";

/// The Keychain's service the workspaces' tokens are kept under, by workspace ID.
pub const KEYCHAIN_SERVICE: &str = "todai.notion";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// How long to wait when Notion asks to slow down without telling for how long.
const DEFAULT_RATE_LIMIT_WAIT: SignedDuration = SignedDuration::from_secs(60);

/// What Notion answered to a request for a page, a database...
#[derive(Debug, Clone, PartialEq)]
pub enum Answer {
    Found(Value),
    /// Missing, deleted, or out of the token's reach: Notion can't tell these apart. With what it said.
    NoAccess(String),
}

/// Notion's API, shared by all the workspaces: each one asks with its own token.
pub struct Notion {
    http: reqwest::Client,
    endpoint: String,
    /// By workspace ID, as read from the Keychain (or just kept there).
    tokens: Mutex<HashMap<String, String>>,
    /// By workspace ID, until when Notion asked to wait.
    rate_limited: Mutex<HashMap<String, Timestamp>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Why Notion didn't answer with what was asked.
#[derive(Debug, PartialEq)]
enum Refusal {
    /// It refused the token.
    Unauthorized,
    Error(IntegrationProblem),
}

impl Notion {
    pub fn new() -> errors::Result<Self> {
        Self::with_endpoint(ENDPOINT)
    }

    pub(crate) fn with_endpoint(endpoint: &str) -> errors::Result<Self> {
        let http = reqwest::Client::builder()
            .user_agent(concat!("todai/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|error| {
                TodaiError::CommandError(format!("Could not set up Notion's client: {error}"))
            })?;
        Ok(Self {
            http,
            endpoint: endpoint.trim_end_matches('/').into(),
            tokens: Mutex::default(),
            rate_limited: Mutex::default(),
        })
    }

    /// Who `token` belongs to, as Notion tells: checks it before it's kept, e.g. when connecting a workspace.
    pub async fn account_of(&self, token: &str) -> Result<NotionAccount, IntegrationProblem> {
        let answer = match self.send(token, "users/me").await {
            Ok(answer) => answer,
            Err(Refusal::Unauthorized) => return Err(IntegrationProblem::TokenRefused),
            Err(Refusal::Error(problem)) => return Err(problem),
        };
        account(answer)
    }

    /// Who the token of the workspace `workspace_id` belongs to, as Notion tells: which also checks that it still accepts
    /// it.
    pub async fn account(
        &self,
        secrets: &impl SecretStore,
        workspace_id: &str,
    ) -> Result<NotionAccount, IntegrationProblem> {
        account(self.get(secrets, workspace_id, "users/me").await?)
    }

    /// Remembers `token` as the one of the workspace `workspace_id`, as just kept in the Keychain.
    pub fn remember_token(&self, workspace_id: &str, token: &str) {
        lock(&self.tokens).insert(workspace_id.into(), token.into());
        lock(&self.rate_limited).remove(workspace_id);
    }

    /// Forgets the token of the workspace `workspace_id`, e.g. once disconnected from Notion.
    pub fn forget(&self, workspace_id: &str) {
        lock(&self.tokens).remove(workspace_id);
        lock(&self.rate_limited).remove(workspace_id);
    }

    /// Asks Notion for `path` (e.g. `pages/<id>`) with the token of the workspace `workspace_id`.
    pub(crate) async fn get(
        &self,
        secrets: &impl SecretStore,
        workspace_id: &str,
        path: &str,
    ) -> Result<Answer, IntegrationProblem> {
        if let Some(reset_at) = self.rate_limited_until(workspace_id, Timestamp::now()) {
            return Err(IntegrationProblem::RateLimited { reset_at });
        }
        let mut known = lock(&self.tokens).get(workspace_id).cloned();
        loop {
            let from_memory = known.is_some();
            let token = match known.take() {
                Some(token) => token,
                None => {
                    let token = secrets
                        .get(workspace_id)
                        .await
                        .map_err(|error| IntegrationProblem::failed(error.to_string()))?
                        .ok_or(IntegrationProblem::TokenMissing)?;
                    lock(&self.tokens).insert(workspace_id.into(), token.clone());
                    token
                }
            };
            match self.send(&token, path).await {
                Ok(answer) => return Ok(answer),
                // Maybe replaced in the Keychain since it was read: asked for again, once
                Err(Refusal::Unauthorized) => {
                    lock(&self.tokens).remove(workspace_id);
                    if !from_memory {
                        return Err(IntegrationProblem::TokenRefused);
                    }
                }
                Err(Refusal::Error(problem)) => {
                    if let IntegrationProblem::RateLimited { reset_at } = problem {
                        log::warn!("Notion's rate limit for workspace {workspace_id} was reached, until {reset_at}");
                        lock(&self.rate_limited).insert(workspace_id.into(), reset_at);
                    }
                    return Err(problem);
                }
            }
        }
    }

    fn rate_limited_until(&self, workspace_id: &str, now: Timestamp) -> Option<Timestamp> {
        lock(&self.rate_limited)
            .get(workspace_id)
            .copied()
            .filter(|reset_at| *reset_at > now)
    }

    async fn send(&self, token: &str, path: &str) -> Result<Answer, Refusal> {
        let offline = |error: reqwest::Error| {
            log::warn!("Could not reach Notion: {error}");
            Refusal::Error(IntegrationProblem::Offline)
        };
        let response = self
            .http
            .get(format!("{}/{path}", self.endpoint))
            .bearer_auth(token)
            .header("Notion-Version", API_VERSION)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(offline)?;
        let status = response.status().as_u16();
        let retry_after = retry_after(response.headers());
        let body = response.bytes().await.map_err(offline)?;
        classify(status, retry_after, &body, Timestamp::now())
    }
}

/// `Retry-After`, in seconds.
fn retry_after(headers: &HeaderMap) -> Option<i64> {
    headers.get(RETRY_AFTER)?.to_str().ok()?.trim().parse().ok()
}

/// What Notion says when it refuses a request, e.g. `{"object":"error","status":401,"code":"unauthorized",...}`.
#[derive(Deserialize, Default)]
struct ErrorBody {
    #[serde(default)]
    code: String,
    #[serde(default)]
    message: String,
    #[serde(default)]
    additional_data: Option<AdditionalData>,
}

#[derive(Deserialize)]
struct AdditionalData {
    /// In whole seconds, as a string.
    retry_after: Option<String>,
}

/// What Notion's answer `body`, with HTTP `status`, means.
fn classify(
    status: u16,
    retry_after: Option<i64>,
    body: &[u8],
    now: Timestamp,
) -> Result<Answer, Refusal> {
    let error = || serde_json::from_slice::<ErrorBody>(body).unwrap_or_default();
    match status {
        200..=299 => serde_json::from_slice(body)
            .map(Answer::Found)
            .map_err(|error| {
                Refusal::Error(IntegrationProblem::failed(format!(
                    "Notion answered something unexpected: {error}"
                )))
            }),
        401 => Err(Refusal::Unauthorized),
        403 | 404 => Ok(Answer::NoAccess(error().message)),
        // `rate_limited`, or `service_overload` (529): to retry the same way
        429 | 529 => {
            let wait = retry_after
                .or_else(|| error().additional_data?.retry_after?.trim().parse().ok())
                .filter(|seconds| *seconds > 0)
                .map_or(DEFAULT_RATE_LIMIT_WAIT, SignedDuration::from_secs);
            Err(Refusal::Error(IntegrationProblem::RateLimited {
                reset_at: now.checked_add(wait).unwrap_or(Timestamp::MAX),
            }))
        }
        500..=599 => {
            log::warn!("Notion failed to answer: HTTP {status}");
            Err(Refusal::Error(IntegrationProblem::Offline))
        }
        _ => {
            let error = error();
            Err(Refusal::Error(IntegrationProblem::failed(format!(
                "Notion answered with HTTP {status}{}: {}",
                if error.code.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", error.code)
                },
                error.message
            ))))
        }
    }
}

/// A user, as `users/me` tells: the creator of a personal access token, or for other tokens, a bot (owned by a user,
/// or by its workspace).
#[derive(Deserialize)]
struct RawUser {
    id: String,
    name: Option<String>,
    #[serde(rename = "type")]
    kind: Option<String>,
    person: Option<RawPerson>,
    bot: Option<RawBot>,
}

#[derive(Deserialize)]
struct RawPerson {
    email: Option<String>,
}

#[derive(Deserialize)]
struct RawBot {
    owner: Option<RawOwner>,
    workspace_name: Option<String>,
}

#[derive(Deserialize)]
struct RawOwner {
    user: Option<Box<RawUser>>,
}

/// Whose token it is, from what `users/me` answered: the person behind it, if Notion tells.
fn account(answer: Answer) -> Result<NotionAccount, IntegrationProblem> {
    let unknown = || IntegrationProblem::failed("Notion didn't tell who the token belongs to");
    let Answer::Found(value) = answer else {
        return Err(unknown());
    };
    let RawUser {
        id,
        name,
        kind,
        person,
        bot,
    } = serde_json::from_value(value).map_err(|_| unknown())?;
    let workspace_name = bot.as_ref().and_then(|bot| bot.workspace_name.clone());
    // A bot owned by a user: that user's
    let owner = bot
        .and_then(|bot| bot.owner?.user)
        .filter(|_| kind.as_deref() == Some("bot"));
    let (user_id, name, person) = match owner {
        Some(owner) => (owner.id, owner.name, owner.person),
        None => (id, name, person),
    };
    Ok(NotionAccount {
        user_id,
        name: name.filter(|name| !name.is_empty()),
        email: person.and_then(|person| person.email),
        workspace_name,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::{
        secrets::FakeSecrets,
        testing::{ok, FakeServer, Response},
    };

    const NOW: i64 = 1_791_000_000;

    fn now() -> Timestamp {
        Timestamp::from_second(NOW).unwrap()
    }

    fn person() -> Value {
        json!({
            "object": "user", "id": "u1", "name": "Pierre-Yves", "avatar_url": null,
            "type": "person", "person": { "email": "py@example.org" },
        })
    }

    fn me() -> Response {
        ok(person().to_string())
    }

    fn unauthorized() -> Response {
        (
            401,
            Vec::new(),
            r#"{"object":"error","status":401,"code":"unauthorized","message":"API token is invalid."}"#.into(),
        )
    }

    #[tokio::test]
    async fn tells_who_a_token_belongs_to() {
        let server = FakeServer::serve(vec![me()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();

        assert_eq!(
            notion.account_of("ntn_t").await,
            Ok(NotionAccount {
                user_id: "u1".into(),
                name: Some("Pierre-Yves".into()),
                email: Some("py@example.org".into()),
                workspace_name: None,
            })
        );
        let request = &server.requests()[0];
        assert_eq!(
            (request.method.as_str(), request.path.as_str()),
            ("GET", "/users/me")
        );
        assert_eq!(request.header("authorization"), Some("Bearer ntn_t"));
        assert_eq!(request.header("notion-version"), Some(API_VERSION));
    }

    #[tokio::test]
    async fn a_refused_token_is_told_apart() {
        let server = FakeServer::serve(vec![unauthorized()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        assert_eq!(
            notion.account_of("ntn_expired").await,
            Err(IntegrationProblem::TokenRefused)
        );
    }

    #[tokio::test]
    async fn reads_a_workspace_token_from_the_keychain_once() {
        let server = FakeServer::serve(vec![me(), me()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::with("w1", "ntn_kept");

        notion.account(&secrets, "w1").await.unwrap();
        notion.account(&secrets, "w1").await.unwrap();
        assert_eq!(secrets.reads(), 1);
        assert_eq!(
            server.authorizations(),
            ["Bearer ntn_kept", "Bearer ntn_kept"]
        );
    }

    #[tokio::test]
    async fn reads_the_keychain_again_once_notion_refuses_a_known_token() {
        let server = FakeServer::serve(vec![me(), unauthorized(), me()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::with("w1", "ntn_old");

        notion.account(&secrets, "w1").await.unwrap();
        // Replaced in the Keychain meanwhile, e.g. by another todai
        secrets.set("w1", "ntn_new").await.unwrap();
        notion.account(&secrets, "w1").await.unwrap();
        assert_eq!(
            server.authorizations(),
            ["Bearer ntn_old", "Bearer ntn_old", "Bearer ntn_new"]
        );
    }

    #[tokio::test]
    async fn a_token_refused_fresh_from_the_keychain_needs_the_user() {
        let server = FakeServer::serve(vec![unauthorized()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::with("w1", "ntn_expired");
        assert_eq!(
            notion.account(&secrets, "w1").await,
            Err(IntegrationProblem::TokenRefused)
        );
        assert_eq!(server.requests().len(), 1);
    }

    #[tokio::test]
    async fn a_token_missing_from_the_keychain_needs_the_user() {
        let server = FakeServer::serve(Vec::new());
        let notion = Notion::with_endpoint(&server.url).unwrap();
        assert_eq!(
            notion.account(&FakeSecrets::default(), "w1").await,
            Err(IntegrationProblem::TokenMissing)
        );
        assert!(server.requests().is_empty());
    }

    #[tokio::test]
    async fn remembered_and_forgotten_tokens() {
        let server = FakeServer::serve(vec![me()]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::default();

        notion.remember_token("w1", "ntn_just_kept");
        notion.account(&secrets, "w1").await.unwrap();
        assert_eq!(secrets.reads(), 0);
        notion.forget("w1");
        assert_eq!(
            notion.account(&secrets, "w1").await,
            Err(IntegrationProblem::TokenMissing)
        );
    }

    #[tokio::test]
    async fn waits_for_the_rate_limit_to_reset_without_asking_notion() {
        let server = FakeServer::serve(vec![(
            429,
            vec![("retry-after", "600".into())],
            r#"{"object":"error","status":429,"code":"rate_limited","message":"Slow down."}"#
                .into(),
        )]);
        let notion = Notion::with_endpoint(&server.url).unwrap();
        let secrets = FakeSecrets::with("w1", "ntn_t");

        let first = notion.account(&secrets, "w1").await;
        assert!(matches!(first, Err(IntegrationProblem::RateLimited { .. })));
        assert_eq!(notion.account(&secrets, "w1").await, first);
        assert_eq!(server.requests().len(), 1);
    }

    #[tokio::test]
    async fn an_unreachable_notion_is_offline() {
        let notion = Notion::with_endpoint(&FakeServer::unreachable_url()).unwrap();
        assert_eq!(
            notion.account_of("ntn_t").await,
            Err(IntegrationProblem::Offline)
        );
    }

    #[test]
    fn classify_tells_what_notion_answered() {
        let classified =
            |status, retry_after, body: &str| classify(status, retry_after, body.as_bytes(), now());
        let in_a_minute = IntegrationProblem::RateLimited {
            reset_at: now().checked_add(DEFAULT_RATE_LIMIT_WAIT).unwrap(),
        };

        assert_eq!(
            classified(200, None, r#"{"object":"user"}"#),
            Ok(Answer::Found(json!({ "object": "user" })))
        );
        assert_eq!(classified(401, None, ""), Err(Refusal::Unauthorized));
        assert_eq!(
            classified(
                404,
                None,
                r#"{"code":"object_not_found","message":"Could not find page."}"#
            ),
            Ok(Answer::NoAccess("Could not find page.".into()))
        );
        assert_eq!(
            classified(
                403,
                None,
                r#"{"code":"restricted_resource","message":"Nope."}"#
            ),
            Ok(Answer::NoAccess("Nope.".into()))
        );
        assert_eq!(
            classified(429, Some(30), "{}"),
            Err(Refusal::Error(IntegrationProblem::RateLimited {
                reset_at: now().checked_add(SignedDuration::from_secs(30)).unwrap()
            }))
        );
        assert_eq!(
            classified(
                529,
                None,
                r#"{"code":"service_overload","additional_data":{"retry_after":"12"}}"#
            ),
            Err(Refusal::Error(IntegrationProblem::RateLimited {
                reset_at: now().checked_add(SignedDuration::from_secs(12)).unwrap()
            }))
        );
        assert_eq!(
            classified(429, None, "not json"),
            Err(Refusal::Error(in_a_minute))
        );
        assert_eq!(
            classified(502, None, "<html>Bad gateway</html>"),
            Err(Refusal::Error(IntegrationProblem::Offline))
        );
        assert!(matches!(
            classified(400, None, r#"{"code":"validation_error","message":"Bad path."}"#),
            Err(Refusal::Error(IntegrationProblem::Failed { message }))
                if message == "Notion answered with HTTP 400 (validation_error): Bad path."
        ));
        assert!(matches!(
            classified(200, None, "not json"),
            Err(Refusal::Error(IntegrationProblem::Failed { .. }))
        ));
    }

    #[test]
    fn the_account_is_the_person_behind_the_token() {
        let bot_of_a_user = json!({
            "object": "user", "id": "b1", "name": "todai", "type": "bot",
            "bot": {
                "owner": { "type": "user", "user": {
                    "object": "user", "id": "u1", "name": "Pierre-Yves", "type": "person",
                    "person": { "email": "py@example.org" },
                }},
                "workspace_id": "nw1", "workspace_name": "Acme",
            },
        });
        assert_eq!(
            account(Answer::Found(bot_of_a_user)),
            Ok(NotionAccount {
                user_id: "u1".into(),
                name: Some("Pierre-Yves".into()),
                email: Some("py@example.org".into()),
                workspace_name: Some("Acme".into()),
            })
        );
        let bot_of_a_workspace = json!({
            "object": "user", "id": "b2", "name": "Acme sync", "type": "bot",
            "bot": { "owner": { "type": "workspace", "workspace": true }, "workspace_name": "Acme" },
        });
        assert_eq!(
            account(Answer::Found(bot_of_a_workspace)),
            Ok(NotionAccount {
                user_id: "b2".into(),
                name: Some("Acme sync".into()),
                email: None,
                workspace_name: Some("Acme".into()),
            })
        );
        assert!(account(Answer::Found(json!({ "object": "list" }))).is_err());
        assert!(account(Answer::NoAccess("Nope.".into())).is_err());
    }
}
