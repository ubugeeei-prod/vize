//! Optional cooperative operator deadline; publication gates keep their own budgets.
use std::{
    env, thread,
    time::{Duration, Instant},
};

const VARIABLE: &str = "VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS";
const MAX_SECONDS: u64 = 28_800;

pub(crate) struct Budget {
    started: Instant,
    limit: Duration,
}

pub(crate) fn seconds(value: &str) -> Result<u64, String> {
    let invalid = || format!("{VARIABLE} must be an integer from 1 through {MAX_SECONDS}.");
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(invalid());
    }
    let seconds = value.parse::<u64>().map_err(|_| invalid())?;
    if !(1..=MAX_SECONDS).contains(&seconds) {
        return Err(invalid());
    }
    Ok(seconds)
}

pub(crate) fn configured() -> Result<Option<Budget>, String> {
    match env::var(VARIABLE) {
        Ok(value) => Ok(Some(Budget::new(Duration::from_secs(seconds(&value)?)))),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(_) => Err(format!("{VARIABLE} is not valid Unicode.")),
    }
}

pub(crate) fn check(budget: Option<&Budget>) -> Result<(), String> {
    budget.map_or(Ok(()), Budget::check)
}

impl Budget {
    pub(crate) fn new(limit: Duration) -> Self {
        Self {
            started: Instant::now(),
            limit,
        }
    }

    pub(crate) fn check(&self) -> Result<(), String> {
        if self.started.elapsed() >= self.limit {
            Err("Release operator watch budget expired. Preserve the same source PR, immutable pin and run; resume with vp run release --resume <source PR> --pin.".into())
        } else {
            Ok(())
        }
    }

    pub(crate) fn sleep(&self, interval: Duration) -> Result<(), String> {
        self.check()?;
        thread::sleep(interval.min(self.limit.saturating_sub(self.started.elapsed())));
        self.check()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_budget_rejects_invalid_or_extended_deadlines() {
        for value in [
            "",
            "0",
            "28801",
            "-1",
            "+1",
            "1.5",
            " 1",
            "1 ",
            "NaN",
            "18446744073709551616",
        ] {
            assert!(seconds(value).is_err(), "{value:?}");
        }
        assert_eq!(seconds("1").unwrap(), 1);
        assert_eq!(seconds("14400").unwrap(), 14_400);
        assert_eq!(seconds("28800").unwrap(), MAX_SECONDS);
    }

    #[test]
    fn operator_polling_stops_at_the_remaining_budget() {
        let budget = Budget::new(Duration::from_millis(1));
        assert!(budget.sleep(Duration::from_secs(20)).is_err());
        assert!(budget.check().is_err());
    }
}
