//! GitHub's GraphQL API, with the token of a GitHub CLI account: asked for to `gh` when needed, and only kept in memory.

use std::{
    collections::HashMap,
    future::Future,
    str::FromStr,
    sync::{Mutex, MutexGuard, PoisonError},
    time::{Duration, Instant},
};

use jiff::{SignedDuration, Timestamp};
use reqwest::header::{HeaderMap, ACCEPT};
use serde_json::Value;

use super::query::{self, Envelope, RateLimit, MAX_ITEMS};
use crate::{
    errors::{self, TodaiError},
    gh::GhFinder,
    integrations::IntegrationProblem,
    links::{snapshot::ItemResult, GitHubRef},
};

pub const ENDPOINT: &str = "https://api.github.com/graphql";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// How long a token is reused before asking `gh` for it again: it may have been revoked, or refreshed, since.
const TOKEN_TTL: Duration = Duration::from_secs(5 * 60);

/// How long to wait when GitHub asks to slow down without telling until when.
const DEFAULT_RATE_LIMIT_WAIT: SignedDuration = SignedDuration::from_secs(60);

/// Where tokens come from: the GitHub CLI, or a fake one in tests.
pub trait TokenSource: Sync {
    /// The token of the account `login`.
    fn token(&self, login: &str)
        -> impl Future<Output = Result<String, IntegrationProblem>> + Send;
}

impl TokenSource for GhFinder {
    async fn token(&self, login: &str) -> Result<String, IntegrationProblem> {
        let gh = self.get(false).await.ok_or(IntegrationProblem::GhMissing)?;
        gh.token(login).await.map_err(|error| match error {
            TodaiError::GhNotFound => IntegrationProblem::GhMissing,
            // What `gh` says once signed out with `gh auth logout`.
            TodaiError::GhCli(message) if message.contains("no oauth token") => {
                IntegrationProblem::SignedOut {
                    login: login.into(),
                }
            }
            error => IntegrationProblem::failed(error.to_string()),
        })
    }
}

/// GitHub's API, shared by all the workspaces: each one asks with the token of its own account.
pub struct GitHub {
    http: reqwest::Client,
    endpoint: String,
    /// By login, with when `gh` gave it.
    tokens: Mutex<HashMap<String, (String, Instant)>>,
    /// By login, until when GitHub asked to wait.
    rate_limited: Mutex<HashMap<String, Timestamp>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Why GitHub didn't answer with data.
enum Refusal {
    /// It refused the token.
    Unauthorized,
    Error(IntegrationProblem),
}

impl GitHub {
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
                TodaiError::CommandError(format!("Could not set up GitHub's client: {error}"))
            })?;
        Ok(Self {
            http,
            endpoint: endpoint.into(),
            tokens: Mutex::default(),
            rate_limited: Mutex::default(),
        })
    }

    /// The pull requests and issues `refs`, as the account `login` sees them, in the same order.
    pub async fn fetch_items(
        &self,
        tokens: &impl TokenSource,
        login: &str,
        refs: &[GitHubRef],
    ) -> Result<Vec<ItemResult>, IntegrationProblem> {
        let mut results = Vec::with_capacity(refs.len());
        for chunk in refs.chunks(MAX_ITEMS) {
            let envelope = self
                .graphql(tokens, login, &query::items_request(chunk))
                .await?;
            results.extend(query::items(&envelope, chunk.len()));
        }
        Ok(results)
    }

    /// The rate limit of the account `login`, which also checks that GitHub accepts its token.
    pub async fn rate_limit(
        &self,
        tokens: &impl TokenSource,
        login: &str,
    ) -> Result<RateLimit, IntegrationProblem> {
        let envelope = self
            .graphql(tokens, login, &query::rate_limit_request())
            .await?;
        query::rate_limit(&envelope)
            .ok_or_else(|| IntegrationProblem::failed("GitHub didn't tell its rate limit"))
    }

    /// Sends the GraphQL request `body` as the account `login`.
    async fn graphql(
        &self,
        tokens: &impl TokenSource,
        login: &str,
        body: &Value,
    ) -> Result<Envelope, IntegrationProblem> {
        if let Some(reset_at) = self.rate_limited_until(login, Timestamp::now()) {
            return Err(IntegrationProblem::RateLimited { reset_at });
        }
        let mut known = self.known_token(login);
        loop {
            let from_memory = known.is_some();
            let token = match known.take() {
                Some(token) => token,
                None => {
                    let token = tokens.token(login).await?;
                    lock(&self.tokens).insert(login.into(), (token.clone(), Instant::now()));
                    token
                }
            };
            match self.post(&token, body).await {
                Ok(envelope) => return Ok(envelope),
                // Revoked or refreshed since it was known: `gh` may have a newer one.
                Err(Refusal::Unauthorized) => {
                    lock(&self.tokens).remove(login);
                    if !from_memory {
                        return Err(IntegrationProblem::SignedOut {
                            login: login.into(),
                        });
                    }
                }
                Err(Refusal::Error(error)) => {
                    if let IntegrationProblem::RateLimited { reset_at } = error {
                        log::warn!("GitHub's rate limit for {login} was reached, until {reset_at}");
                        lock(&self.rate_limited).insert(login.into(), reset_at);
                    }
                    return Err(error);
                }
            }
        }
    }

    fn known_token(&self, login: &str) -> Option<String> {
        lock(&self.tokens)
            .get(login)
            .filter(|(_, at)| at.elapsed() < TOKEN_TTL)
            .map(|(token, _)| token.clone())
    }

    fn rate_limited_until(&self, login: &str, now: Timestamp) -> Option<Timestamp> {
        lock(&self.rate_limited)
            .get(login)
            .copied()
            .filter(|reset_at| *reset_at > now)
    }

    async fn post(&self, token: &str, body: &Value) -> Result<Envelope, Refusal> {
        let offline = |error: reqwest::Error| {
            log::warn!("Could not reach GitHub: {error}");
            Refusal::Error(IntegrationProblem::Offline)
        };
        let response = self
            .http
            .post(&self.endpoint)
            .bearer_auth(token)
            .header(ACCEPT, "application/json")
            .json(body)
            .send()
            .await
            .map_err(offline)?;
        let status = response.status().as_u16();
        let limits = RateHeaders::from(response.headers());
        let body = response.bytes().await.map_err(offline)?;
        classify(status, &limits, &body, Timestamp::now())
    }
}

/// What GitHub's answer tells about the rate limit.
#[derive(Debug, Default)]
struct RateHeaders {
    /// `x-ratelimit-remaining`.
    remaining: Option<u32>,
    /// `x-ratelimit-reset`: when the limit resets, in seconds since the Unix epoch.
    reset: Option<i64>,
    /// `retry-after`, in seconds: for the secondary rate limits.
    retry_after: Option<i64>,
}

impl From<&HeaderMap> for RateHeaders {
    fn from(headers: &HeaderMap) -> Self {
        fn number<T: FromStr>(headers: &HeaderMap, name: &str) -> Option<T> {
            headers.get(name)?.to_str().ok()?.trim().parse().ok()
        }
        Self {
            remaining: number(headers, "x-ratelimit-remaining"),
            reset: number(headers, "x-ratelimit-reset"),
            retry_after: number(headers, "retry-after"),
        }
    }
}

impl RateHeaders {
    /// Until when to wait, from `now`.
    fn reset_at(&self, now: Timestamp) -> Timestamp {
        let at = match (self.retry_after, self.reset) {
            (Some(seconds), _) => now.checked_add(SignedDuration::from_secs(seconds)).ok(),
            (None, Some(reset)) => Timestamp::from_second(reset).ok(),
            (None, None) => None,
        };
        at.filter(|at| *at > now).unwrap_or_else(|| {
            now.checked_add(DEFAULT_RATE_LIMIT_WAIT)
                .unwrap_or(Timestamp::MAX)
        })
    }
}

/// What GitHub's answer `body`, with HTTP `status`, means.
fn classify(
    status: u16,
    limits: &RateHeaders,
    body: &[u8],
    now: Timestamp,
) -> Result<Envelope, Refusal> {
    let rate_limited = || {
        Refusal::Error(IntegrationProblem::RateLimited {
            reset_at: limits.reset_at(now),
        })
    };
    // E.g. `{"message": "Bad credentials", ...}`.
    let message = || {
        serde_json::from_slice::<Value>(body)
            .ok()
            .and_then(|body| body.get("message")?.as_str().map(str::to_owned))
            .unwrap_or_default()
    };
    match status {
        200..=299 => {
            let envelope: Envelope = serde_json::from_slice(body).map_err(|error| {
                Refusal::Error(IntegrationProblem::failed(format!(
                    "GitHub answered something unexpected: {error}"
                )))
            })?;
            if envelope
                .errors
                .iter()
                .any(|error| error.kind.as_deref() == Some("RATE_LIMITED"))
            {
                return Err(rate_limited());
            }
            if envelope.data.is_none() {
                let reason = envelope.errors.first().map_or("", |error| &error.message);
                return Err(Refusal::Error(IntegrationProblem::failed(format!(
                    "GitHub refused the request: {reason}"
                ))));
            }
            Ok(envelope)
        }
        401 => Err(Refusal::Unauthorized),
        403 | 429
            if status == 429
                || limits.remaining == Some(0)
                || limits.retry_after.is_some()
                || message().to_lowercase().contains("rate limit") =>
        {
            Err(rate_limited())
        }
        500..=599 => {
            log::warn!("GitHub failed to answer: HTTP {status}");
            Err(Refusal::Error(IntegrationProblem::Offline))
        }
        _ => Err(Refusal::Error(IntegrationProblem::failed(format!(
            "GitHub answered with HTTP {status}: {}",
            message()
        )))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        github::testing::FakeTokens,
        testing::{ok, FakeServer, Response},
    };

    const NOW: i64 = 1_791_000_000;

    fn now() -> Timestamp {
        Timestamp::from_second(NOW).unwrap()
    }

    const RATE_LIMIT_BODY: &str = r#"{"data":{"rateLimit":{"limit":5000,"remaining":4990,"resetAt":"2026-10-10T21:00:00Z"}}}"#;

    fn rate_limit_ok() -> Response {
        ok(RATE_LIMIT_BODY)
    }

    fn unauthorized() -> Response {
        (401, Vec::new(), r#"{"message":"Bad credentials"}"#.into())
    }

    #[tokio::test]
    async fn asks_gh_for_a_token_once_then_reuses_it() {
        let server = FakeServer::serve(vec![rate_limit_ok(), rate_limit_ok()]);
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let tokens = FakeTokens::default();

        let rate_limit = github.rate_limit(&tokens, "octocat").await.unwrap();
        assert_eq!((rate_limit.limit, rate_limit.remaining), (5000, 4990));
        github.rate_limit(&tokens, "octocat").await.unwrap();

        assert_eq!(tokens.asked(), 1);
        assert_eq!(server.authorizations(), ["Bearer t1", "Bearer t1"]);
    }

    #[tokio::test]
    async fn asks_gh_again_once_when_github_refuses_a_known_token() {
        let server = FakeServer::serve(vec![rate_limit_ok(), unauthorized(), rate_limit_ok()]);
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let tokens = FakeTokens::default();

        github.rate_limit(&tokens, "octocat").await.unwrap();
        github.rate_limit(&tokens, "octocat").await.unwrap();

        assert_eq!(tokens.asked(), 2);
        assert_eq!(
            server.authorizations(),
            ["Bearer t1", "Bearer t1", "Bearer t2"]
        );
    }

    #[tokio::test]
    async fn a_refused_token_fresh_from_gh_means_signed_out() {
        let server = FakeServer::serve(vec![unauthorized(), rate_limit_ok()]);
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let tokens = FakeTokens::default();

        assert_eq!(
            github.rate_limit(&tokens, "octocat").await,
            Err(IntegrationProblem::SignedOut {
                login: "octocat".into()
            })
        );
        assert_eq!(tokens.asked(), 1);
        assert_eq!(server.authorizations().len(), 1);
    }

    #[tokio::test]
    async fn waits_for_the_rate_limit_to_reset_without_asking_github() {
        let reset = Timestamp::now().as_second() + 600;
        let server = FakeServer::serve(vec![(
            403,
            vec![
                ("x-ratelimit-remaining", "0".into()),
                ("x-ratelimit-reset", reset.to_string()),
            ],
            r#"{"message":"API rate limit exceeded"}"#.into(),
        )]);
        let github = GitHub::with_endpoint(&server.url).unwrap();
        let tokens = FakeTokens::default();
        let limited = Err(IntegrationProblem::RateLimited {
            reset_at: Timestamp::from_second(reset).unwrap(),
        });

        assert_eq!(github.rate_limit(&tokens, "octocat").await, limited);
        assert_eq!(github.rate_limit(&tokens, "octocat").await, limited);
        assert_eq!(server.authorizations().len(), 1);
    }

    #[tokio::test]
    async fn an_unreachable_github_is_offline() {
        let github = GitHub::with_endpoint(&FakeServer::unreachable_url()).unwrap();
        assert_eq!(
            github.rate_limit(&FakeTokens::default(), "octocat").await,
            Err(IntegrationProblem::Offline)
        );
    }

    #[tokio::test]
    async fn fetches_items_in_batches() {
        let refs: Vec<GitHubRef> = (1..=MAX_ITEMS as u32 + 1)
            .map(|number| GitHubRef {
                owner: "o".into(),
                repo: "r".into(),
                number,
            })
            .collect();
        // Nothing found: each one is then without access.
        let server = FakeServer::serve(vec![ok(r#"{"data":{}}"#), ok(r#"{"data":{}}"#)]);
        let github = GitHub::with_endpoint(&server.url).unwrap();

        let results = github
            .fetch_items(&FakeTokens::default(), "octocat", &refs)
            .await
            .unwrap();
        assert_eq!(results.len(), refs.len());
        assert!(results.iter().all(|result| *result == ItemResult::NoAccess));
        assert_eq!(server.authorizations().len(), 2);
    }

    fn classified(status: u16, limits: &RateHeaders, body: &str) -> Result<(), IntegrationProblem> {
        match classify(status, limits, body.as_bytes(), now()) {
            Ok(_) => Ok(()),
            Err(Refusal::Unauthorized) => Err(IntegrationProblem::SignedOut {
                login: String::new(),
            }),
            Err(Refusal::Error(error)) => Err(error),
        }
    }

    #[test]
    fn classify_tells_what_github_answered() {
        let none = RateHeaders::default();
        let in_a_minute = IntegrationProblem::RateLimited {
            reset_at: now().checked_add(DEFAULT_RATE_LIMIT_WAIT).unwrap(),
        };
        assert_eq!(classified(200, &none, RATE_LIMIT_BODY), Ok(()));
        // Partial data: some items not found.
        assert_eq!(
            classified(
                200,
                &none,
                r#"{"data":{"l0":null},"errors":[{"type":"NOT_FOUND","message":"Could not resolve","path":["l0"]}]}"#
            ),
            Ok(())
        );
        assert_eq!(
            classified(401, &none, r#"{"message":"Bad credentials"}"#),
            Err(IntegrationProblem::SignedOut {
                login: String::new()
            })
        );
        assert_eq!(
            classified(
                200,
                &none,
                r#"{"errors":[{"type":"RATE_LIMITED","message":"API rate limit exceeded"}]}"#
            ),
            Err(in_a_minute.clone())
        );
        assert_eq!(
            classified(
                403,
                &RateHeaders {
                    retry_after: Some(30),
                    ..RateHeaders::default()
                },
                r#"{"message":"You have exceeded a secondary rate limit."}"#
            ),
            Err(IntegrationProblem::RateLimited {
                reset_at: now().checked_add(SignedDuration::from_secs(30)).unwrap()
            })
        );
        assert_eq!(
            classified(
                403,
                &none,
                r#"{"message":"You have exceeded a secondary rate limit."}"#
            ),
            Err(in_a_minute)
        );
        assert_eq!(
            classified(502, &none, "<html>Bad gateway</html>"),
            Err(IntegrationProblem::Offline)
        );
        assert!(matches!(
            classified(403, &none, r#"{"message":"Resource not accessible by integration"}"#),
            Err(IntegrationProblem::Failed { message }) if message.contains("Resource not accessible")
        ));
        assert!(matches!(
            classified(200, &none, r#"{"errors":[{"message":"Parse error on \"}\""}]}"#),
            Err(IntegrationProblem::Failed { message }) if message.contains("Parse error")
        ));
        assert!(matches!(
            classified(200, &none, "not json"),
            Err(IntegrationProblem::Failed { .. })
        ));
    }
}
