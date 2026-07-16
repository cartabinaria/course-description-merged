// SPDX-License-Identifier: GPL-3.0-or-later

use log::debug;
use reqwest::Error;
use reqwest::blocking::{Response, get};
use std::{thread::sleep, time::Duration};

/// Number of attempts for transient HTTP/network failures.
const RETRY_ATTEMPTS: usize = 2;
/// Base delay used for exponential backoff across retries.
const RETRY_BASE_DELAY_MS: u64 = 250;

/// Returns `true` for network/status failures that are usually temporary.
fn is_retryable_request_error(err: &Error) -> bool {
    err.is_timeout()
        || err.is_connect()
        || err.status().is_some_and(|status| {
            status.as_u16() == 408 || status.as_u16() == 429 || status.is_server_error()
        })
}

/// GET request with status check and eventual retry.
fn request_with_retry(url: &str, attempt: usize, delay_ms: u64) -> Result<Response, Error> {
    match get(url).and_then(|res| res.error_for_status()) {
        Ok(res) => Ok(res),
        Err(err) if (attempt > 0) && is_retryable_request_error(&err) => {
            debug!(
                "[request_url={url} retries_left={} backoff_ms={delay_ms}] Retrying request due to transient error: {err}",
                attempt - 1,
            );

            sleep(Duration::from_millis(delay_ms));

            request_with_retry(url, attempt - 1, delay_ms * 2)
        }
        Err(err) => Err(err),
    }
}

/// GET request with status check and exponential backoff retries.
pub fn get_status_checked_with_retry(url: &str) -> Result<Response, Error> {
    request_with_retry(url, RETRY_ATTEMPTS, RETRY_BASE_DELAY_MS)
}
