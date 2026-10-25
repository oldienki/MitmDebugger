#![allow(unused)]

pub enum FlowsRange {
    /// One exact flow.
    Exact(u64),
    /// Specified count of flows from the latest.
    Count(usize),
    /// Specified count of flows from the specified one (older).
    After(usize, u64),
    /// Specified count of flows before the specified one (newer).
    Before(usize, u64),
}

pub enum ContentLevel {
    /// Just minimal request information.
    Minimal,
    /// Minimal request information + notes.
    Notes,
    /// All requests and responses info + TLS.
    Headers,
    /// All requests and responses info + TLS + body + notes.
    Full,
}

pub enum Flow {
    Http(HttpFlow),
}

pub struct HttpFlow {
    pub id: u64,
    pub timestamp: u64,
    pub request: HttpRequest,
    pub tls: Option<TlsSecurity>,
    pub modified_request: Option<Box<HttpRequest>>,
    pub response: Option<Box<HttpResponse>>,
    pub modified_response: Option<Box<HttpResponse>>,
    pub notes: Option<Vec<(String, String)>>,
    pub server_error: Option<String>,
}

pub struct HttpRequest {
    pub host: String,
    pub port: u16,
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}
pub struct HttpResponse {
    pub version: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

pub struct TlsSecurity {}

#[derive(Debug, thiserror::Error)]
pub enum FlowsError {
    #[error("{0}")]
    General(String),
}
