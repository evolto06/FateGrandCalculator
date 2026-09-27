use std::io::{ErrorKind, Read};
use std::thread;
use std::time::Duration;

use reqwest::blocking::{Client, Response};
use reqwest::header::{CONTENT_LENGTH, RETRY_AFTER};
use serde_json::Value;

const BASIC_EXPORT_URL: &str = "https://api.atlasacademy.io/export/NA/basic_servant.json";
const MAX_RESPONSE_BYTES: usize = 32 * 1024 * 1024;
const MAX_ATTEMPTS: usize = 3;

pub struct AtlasClient {
    client: Client,
}

impl AtlasClient {
    pub fn new() -> Result<Self, String> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .user_agent(concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION")
            ))
            .build()
            .map_err(|error| format!("Could not initialize the Atlas Academy client: {error}"))?;
        Ok(Self { client })
    }

    pub fn fetch_basic_export(&self) -> Result<Value, String> {
        for attempt in 1..=MAX_ATTEMPTS {
            match self.fetch_once() {
                Ok(payload) => return Ok(payload),
                Err(FetchError {
                    message,
                    retryable,
                    retry_after,
                }) => {
                    if !retryable || attempt == MAX_ATTEMPTS {
                        return Err(format!(
                            "Atlas Academy servant data could not be downloaded after {attempt} attempt(s): {message}"
                        ));
                    }
                    let delay =
                        retry_after.unwrap_or_else(|| Duration::from_millis(350 * attempt as u64));
                    thread::sleep(delay);
                }
            }
        }

        Err("Atlas Academy servant data download ended unexpectedly.".into())
    }

    fn fetch_once(&self) -> Result<Value, FetchError> {
        let response = self
            .client
            .get(BASIC_EXPORT_URL)
            .send()
            .map_err(|error| FetchError {
                message: format!("network request failed: {error}"),
                retryable: true,
                retry_after: None,
            })?;

        let response = checked_response(response)?;
        let mut body = Vec::new();
        response
            .take((MAX_RESPONSE_BYTES + 1) as u64)
            .read_to_end(&mut body)
            .map_err(|error| FetchError {
                message: if error.kind() == ErrorKind::TimedOut {
                    "the response timed out while downloading".into()
                } else {
                    format!("could not read the response: {error}")
                },
                retryable: true,
                retry_after: None,
            })?;

        if body.len() > MAX_RESPONSE_BYTES {
            return Err(FetchError {
                message: format!(
                    "the Atlas export exceeded the {} MiB safety limit",
                    MAX_RESPONSE_BYTES / (1024 * 1024)
                ),
                retryable: false,
                retry_after: None,
            });
        }

        serde_json::from_slice(&body).map_err(|error| FetchError {
            message: format!("Atlas returned invalid JSON: {error}"),
            retryable: false,
            retry_after: None,
        })
    }
}

fn checked_response(response: Response) -> Result<Response, FetchError> {
    let status = response.status();
    if status.is_success() {
        if response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok())
            .is_some_and(|length| length > MAX_RESPONSE_BYTES)
        {
            return Err(FetchError {
                message: format!(
                    "the Atlas export exceeded the {} MiB safety limit",
                    MAX_RESPONSE_BYTES / (1024 * 1024)
                ),
                retryable: false,
                retry_after: None,
            });
        }
        return Ok(response);
    }

    let retryable = status.as_u16() == 429 || status.is_server_error();
    let retry_after = response
        .headers()
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(|seconds| Duration::from_secs(seconds.clamp(1, 30)));
    let message = match status.as_u16() {
        403 => "Atlas Academy rejected the export request (HTTP 403).".into(),
        404 => "Atlas Academy's servant export was not found (HTTP 404).".into(),
        429 => "Atlas Academy is rate-limiting requests (HTTP 429).".into(),
        code => format!("Atlas Academy returned HTTP {code}."),
    };
    Err(FetchError {
        message,
        retryable,
        retry_after,
    })
}

struct FetchError {
    message: String,
    retryable: bool,
    retry_after: Option<Duration>,
}
