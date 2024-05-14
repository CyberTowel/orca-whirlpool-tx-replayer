use rand::Rng;
use std::{fmt::Display, time::Duration};

pub async fn retry<F, Fut, T, E>(mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut backoff = 100u64; // Start with 100 milliseconds
    let max_backoff = 3_000; // Maximum backoff of 30 seconds

    loop {
        match operation().await {
            Ok(value) => return Ok(value), // Return the successful value

            Err(err) => {
                // println!("Error: {:?}", err);
                let jitter = rand::thread_rng().gen_range(0..backoff); // Add some jitter
                tokio::time::sleep(Duration::from_millis(backoff + jitter)).await; // Wait before retrying

                backoff = backoff.saturating_mul(2).min(max_backoff); // Exponential backoff
            }
            _ => {}
        }
    }
}
