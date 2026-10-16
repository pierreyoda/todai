//! Fakes for tests: a provider's API.

use std::{
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

/// A response: its status, extra headers, and body.
pub type Response = (u16, Vec<(&'static str, String)>, String);

pub fn ok(body: impl Into<String>) -> Response {
    (200, Vec::new(), body.into())
}

/// A request a `FakeServer` received.
#[derive(Debug, Clone, Default)]
pub struct Request {
    pub method: String,
    /// With its query, e.g. `/v1/users/me`.
    pub path: String,
    /// Lowercase names, with their value.
    pub headers: Vec<(String, String)>,
}

impl Request {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(header, _)| header == name)
            .map(|(_, value)| value.as_str())
    }
}

/// A local HTTP server answering each request with the next of the responses it was given, whatever its path: a fake
/// of a provider's API.
pub struct FakeServer {
    /// Where it listens, e.g. `http://127.0.0.1:61234`.
    pub url: String,
    requests: Arc<Mutex<Vec<Request>>>,
}

impl FakeServer {
    pub fn serve(responses: Vec<Response>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let recorded = requests.clone();
        std::thread::spawn(move || {
            for ((status, headers, body), stream) in responses.into_iter().zip(listener.incoming())
            {
                let mut stream = stream.unwrap();
                let request = read_request(&mut stream);
                recorded.lock().unwrap().push(request);
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
        Self { url, requests }
    }

    /// Where nothing listens.
    pub fn unreachable_url() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        format!("http://{}", listener.local_addr().unwrap())
    }

    /// The requests received, in order.
    pub fn requests(&self) -> Vec<Request> {
        self.requests.lock().unwrap().clone()
    }

    /// The `Authorization` header of each request.
    pub fn authorizations(&self) -> Vec<String> {
        self.requests()
            .iter()
            .map(|request| request.header("authorization").unwrap_or_default().into())
            .collect()
    }
}

/// Reads a whole request.
fn read_request(stream: &mut TcpStream) -> Request {
    let mut reader = BufReader::new(stream);
    let mut request = Request::default();
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    let mut parts = line.split_whitespace();
    request.method = parts.next().unwrap_or_default().into();
    request.path = parts.next().unwrap_or_default().into();
    let mut length = 0;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        let (name, value) = line.split_once(':').unwrap_or((line, ""));
        let (name, value) = (name.to_ascii_lowercase(), value.trim().to_owned());
        if name == "content-length" {
            length = value.parse().unwrap();
        }
        request.headers.push((name, value));
    }
    reader.read_exact(&mut vec![0; length]).unwrap();
    request
}
