use rand::Rng;
use std::{time::Duration};

pub async fn retry_blocks<F, Fut, T, E>(mut operation: F) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut backoff = 100u64; // Start with 100 milliseconds
    let max_backoff = 3_000; // Maximum backoff of 30 seconds

    let mut rety_counts = 0;

    loop {
        match operation().await {
            Ok(value) => return Ok(value), // Return the successful value

            Err(err) => {
                rety_counts += 1;

                if rety_counts > 20 {
                    // println!(
                    //     "Block {:?} has been tried 20 times, giving up",
                    //     block_numbner
                    // );
                    return Err(err);
                }

                let jitter = rand::thread_rng().gen_range(0..backoff); // Add some jitter
                tokio::time::sleep(Duration::from_millis(backoff + jitter)).await; // Wait before retrying

                backoff = backoff.saturating_mul(2).min(max_backoff); // Exponential backoff
            }
        }
    }
}
