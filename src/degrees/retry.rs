// SPDX-License-Identifier: GPL-3.0-or-later

use log::debug;
use reqwest::blocking::{Response, get};
use std::{thread::sleep, time::Duration};

/// Number of attempts for transient HTTP/network failures.
const RETRY_ATTEMPTS: usize = 3;
/// Base delay used for exponential backoff across retries.
const RETRY_BASE_DELAY_MS: u64 = 250;

/// Returns `true` for network/status failures that are usually temporary.
fn is_retryable_request_error(err: &reqwest::Error) -> bool {
    if err.is_timeout() || err.is_connect() {
        return true;
    }

    err.status().is_some_and(|status| {
        status.as_u16() == 408 || status.as_u16() == 429 || status.is_server_error()
    })
}

/// GET request with status check and exponential backoff retries.
pub fn get_status_checked_with_retry(url: &str) -> Result<Response, reqwest::Error> {
    let mut delay_ms = RETRY_BASE_DELAY_MS;

    for attempt in 1..=RETRY_ATTEMPTS {
        match get(url).and_then(|res| res.error_for_status()) {
            Ok(res) => return Ok(res),
            Err(err) => {
                let should_retry = attempt < RETRY_ATTEMPTS && is_retryable_request_error(&err);
                if !should_retry {
                    return Err(err);
                }

                debug!(
                    "[request_url={url} attempt={attempt} max_attempts={RETRY_ATTEMPTS} backoff_ms={delay_ms}] Retrying request due to transient error: {err}"
                );
                sleep(Duration::from_millis(delay_ms));
                delay_ms *= 2;
            }
        }
    }

    unreachable!("retry loop should have returned before this point")
}
