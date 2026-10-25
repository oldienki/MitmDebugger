
// FlowsSnapshot is a trait that represents a snapshot of the flows.
// It fixes only the last flow id, not the content, so pending flows may become
// completed after some time. It also means that no newer flows will be
// accessible inside the snapshot.
// It presents the flows by id in descending order (newer -> older).
#[async_trait::async_trait]
pub trait FlowsSnapshot {
    fn latest() -> u64;
    async fn list(&self, range: FlowsRange, content_level: ContentLevel) -> Result<Vec<Flow>,FlowsViewError>;
}

enum FlowsRange {
    Exact(u64),
    Count(usize),
    After(usize,u64),
    Before(usize,u64),
}

enum ContentLevel {
    Minimal, // just minimal request information
    Notes, // just minimal request information + notes
    Headers, // all requests and responses info + Tls
    Full, // all requests and responses info + Tls + body + notes
}

enum Flow {
    Http(HttpFlow)
}

struct HttpFlow {
    pub id: u64,
    pub timestamp: u64,
    pub request: HttpRequest,
    pub tls: Option<TlsSecurity>,
    pub modified_request: Option<Box<HttpRequest>>,
    pub response: Option<Box<HttpResponse>>,
    pub modified_response: Option<Box<HttpResponse>>,
    pub notes: Option<Vec<(String,String)>>,
    pub server_error: Option<String>,
}

struct HttpRequest {
    pub host: String,
    pub port: u16,
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}
struct HttpResponse {
    pub version: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

pub struct TlsSecurity {
}

pub struct FlowsView {
    snapshot: Box<dyn FlowsSnapshot>,
}

impl FlowsView {
    pub fn list(&self, range: FlowsRange, content_level: ContentLevel) -> Result<Vec<Flow>,FlowsViewError> {
        tokio::
        Ok(vec![])
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FlowsViewError {
    #[error("{0}")]
    General(String),
}

