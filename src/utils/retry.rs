use anyhow::Result;
use std::time::Duration;

/// Maximum number of attempts.
const MAX_ATTEMPTS: u32 = 3;

/// Exponential backoff delays in seconds: 1s → 2s → (4s used only if retrying 4 times).
const BACKOFF_START_SECS: u64 = 1;

/// Run a fallible operation with exponential backoff retries.
///
/// Attempts the closure up to `MAX_ATTEMPTS` times. After a failure, waits
/// 1s, 2s, then gives up. The last error is returned on final failure.
pub fn with_retry<T, F>(mut f: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    let mut delay_secs = BACKOFF_START_SECS;
    let mut last_error = None;

    for attempt in 0..MAX_ATTEMPTS {
        match f() {
            Ok(value) => return Ok(value),
            Err(err) => {
                if attempt + 1 >= MAX_ATTEMPTS {
                    return Err(err);
                }
                eprintln!(
                    "  ⟳ {} (retry {}/{})",
                    crate::utils::colors::Colors::warning(
                        &format!("Retrying after error: {err}")
                    ),
                    attempt + 2,
                    MAX_ATTEMPTS
                );
                last_error = Some(err);
                std::thread::sleep(Duration::from_secs(delay_secs));
                delay_secs *= 2;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Operation failed")))
}