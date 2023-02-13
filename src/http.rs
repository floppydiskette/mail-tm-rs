use anyhow::Error;
use isahc::http::header::{AUTHORIZATION, CONTENT_TYPE};
use isahc::http::{HeaderMap, StatusCode};
use isahc::{HttpClient, HttpClientBuilder};

use crate::error::HttpError;
use crate::USER_AGENT;

pub struct Client {
    headers: HeaderMap,
    builder: HttpClientBuilder,
}

impl Client {
    pub fn new() -> Result<Client, Error> {
        let client = Client { // TODO: This can be cached
            headers: get_headers()?,
            builder: HttpClientBuilder::new(),
        };
        Ok(client)
    }

    pub fn with_auth(mut self, token: &str) -> Result<Client, Error> {
        self.headers
            .insert(AUTHORIZATION, format!("Bearer {}", token).parse()?);
        Ok(self)
    }

    pub fn build(self) -> Result<HttpClient, Error> {
        Ok(self.builder.default_headers(self.headers.iter())
            .default_header("User-Agent", USER_AGENT).build()?)
    }
}

pub fn get_headers() -> Result<HeaderMap, Error> {
    let mut header_map = HeaderMap::new();
    header_map.insert("User-Agent", USER_AGENT.parse()?);
    header_map.insert("Origin", "https://mail.tm".parse()?); // TODO test if needed
    header_map.insert("TE", "Trailers".parse()?); // TODO test if needed
    header_map.insert(CONTENT_TYPE, "application/json;charset=utf-8".parse()?); //TODO memoize me
    Ok(header_map)
}

pub fn check_response_status(status: &StatusCode, res: &str) -> Result<(), Error> {
    if !status.is_success() {
        return Err(HttpError::Status(status.as_u16(), res.to_string()).into());
    }
    Ok(())
}
