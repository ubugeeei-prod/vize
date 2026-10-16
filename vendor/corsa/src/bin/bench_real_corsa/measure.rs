use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use corsa::{Result, fast::SmallVec};

use crate::{
    profile::{BenchProfiler, ScenarioProfileRow, summarize},
    stats::Stats,
};

pub async fn measure<F, Fut>(iterations: usize, mut f: F) -> Result<Stats>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let mut samples = SmallVec::<[Duration; 32]>::new();
    let mut index = 0_usize;
    while index < iterations {
        let started = Instant::now();
        f().await?;
        samples.push(started.elapsed());
        index += 1;
    }
    Ok(Stats::from_samples(samples))
}

pub async fn measure_warm<F, Fut>(iterations: usize, mut f: F) -> Result<Stats>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    f().await?;
    measure(iterations, f).await
}

pub async fn measure_profiled<F, Fut>(
    iterations: usize,
    profiler: &BenchProfiler,
    mut f: F,
) -> Result<(Stats, SmallVec<[ScenarioProfileRow; 32]>)>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    let mut samples = SmallVec::<[Duration; 32]>::new();
    let mut profile_samples = BTreeMap::<
        (corsa::fast::CompactString, corsa::fast::CompactString),
        SmallVec<[Duration; 32]>,
    >::new();
    let mut index = 0_usize;
    while index < iterations {
        profiler.clear();
        let started = Instant::now();
        f().await?;
        samples.push(started.elapsed());
        for (key, duration) in profiler.drain_iteration_totals() {
            profile_samples.entry(key).or_default().push(duration);
        }
        index += 1;
    }
    Ok((Stats::from_samples(samples), summarize(profile_samples)))
}

pub async fn measure_warm_profiled<F, Fut>(
    iterations: usize,
    profiler: &BenchProfiler,
    mut f: F,
) -> Result<(Stats, SmallVec<[ScenarioProfileRow; 32]>)>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    profiler.clear();
    f().await?;
    let _ = profiler.drain_iteration_totals();
    measure_profiled(iterations, profiler, f).await
}
