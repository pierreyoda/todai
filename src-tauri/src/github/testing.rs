//! Fakes of GitHub and of the GitHub CLI's tokens, for tests.

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use super::{GitHubError, TokenSource};

/// Gives `t1`, `t2`... each time it's asked.
#[derive(Default)]
pub struct FakeTokens {
    asked: AtomicUsize,
}

impl TokenSource for FakeTokens {
    async fn token(&self, _login: &str) -> Result<String, GitHubError> {
        let asked = self.asked.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(format!("t{asked}"))
    }
}

impl FakeTokens {
    pub fn asked(&self) -> usize {
        self.asked.load(Ordering::SeqCst)
    }
}

/// A response: its status, extra headers, and body.
pub type Response = (u16, Vec<(&'static str, String)>, String);

pub fn ok(body: impl Into<String>) -> Response {
    (200, Vec::new(), body.into())
}

/// A local HTTP server answering each request with the next of the responses it was given.
pub struct FakeGitHub {
    pub endpoint: String,
    /// The `Authorization` header of each request.
    authorizations: Arc<Mutex<Vec<String>>>,
}

impl FakeGitHub {
    pub fn serve(responses: Vec<Response>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/graphql", listener.local_addr().unwrap());
        let authorizations = Arc::new(Mutex::new(Vec::new()));
        let recorded = authorizations.clone();
        std::thread::spawn(move || {
            for ((status, headers, body), stream) in responses.into_iter().zip(listener.incoming())
            {
                let mut stream = stream.unwrap();
                let authorization = read_authorization(&mut stream);
                recorded.lock().unwrap().push(authorization);
                let mut response = format!(
                    "HTTP/1.1 {status} Status\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n",
                    body.len()
                );
                for (name, value) in headers {
                    response.push_str(&format!("{name}: {value}\r\n"));
                }
                response.push_str("\r\n");
                response.push_str(&body);
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        Self {
            endpoint,
            authorizations,
        }
    }

    /// Where nothing listens.
    pub fn unreachable_endpoint() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}/graphql", listener.local_addr().unwrap())
    }

    pub fn authorizations(&self) -> Vec<String> {
        self.authorizations.lock().unwrap().clone()
    }
}

/// Reads a whole request, returning its `Authorization` header.
fn read_authorization(stream: &mut TcpStream) -> String {
    let mut reader = BufReader::new(stream);
    let (mut authorization, mut length) = (String::new(), 0);
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':').unwrap_or((line, ""));
        match name.to_ascii_lowercase().as_str() {
            "authorization" => authorization = value.trim().into(),
            "content-length" => length = value.trim().parse().unwrap(),
            _ => {}
        }
    }
    reader.read_exact(&mut vec![0; length]).unwrap();
    authorization
}
