use std::time::{Duration, Instant};

use corsa::{Result, fast::SmallVec};

use crate::stats::Stats;

pub async fn measure_with_warmup<F, Fut>(
    warmups: usize,
    iterations: usize,
    mut f: F,
) -> Result<Stats>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let mut index = 0_usize;
    while index < warmups {
        f().await?;
        index += 1;
    }
    let mut samples = SmallVec::<[Duration; 32]>::new();
    let mut iteration = 0_usize;
    while iteration < iterations {
        let started = Instant::now();
        f().await?;
        samples.push(started.elapsed());
        iteration += 1;
    }
    Ok(Stats::from_samples(samples))
}
