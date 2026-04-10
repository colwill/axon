// JSON-lines protocol for agent communication.
//
// Input:  one JSON object per line on stdin
// Output: one JSON object per line on stdout

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Request {
    pub action: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub axon: String,
}

#[derive(Serialize)]
pub struct EncodeResponse {
    pub ok: bool,
    pub axon: String,
    pub annotation: String,
    pub input_tokens: usize,
    pub axon_tokens: usize,
    pub savings_pct: usize,
}

#[derive(Serialize)]
pub struct DecodeResponse {
    pub ok: bool,
    pub text: String,
}

#[derive(Serialize)]
pub struct TokensResponse {
    pub ok: bool,
    pub tokens: usize,
}

#[derive(Serialize)]
pub struct CompressResponse {
    pub ok: bool,
    pub encoded: String,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
    pub ratio: f64,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub ok: bool,
    pub error: String,
}

impl ErrorResponse {
    pub fn new(msg: &str) -> Self {
        Self {
            ok: false,
            error: msg.to_string(),
        }
    }
}
