//! Optional cooperative operator deadline; publication gates keep their own budgets.
use std::{
    env,
    process::Command,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const VARIABLE: &str = "VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS";
const MAX_SECONDS: u64 = 28_800;
const DEADLINE: &str = "VIZE_RELEASE_OPERATOR_DEADLINE_UNIX_MILLIS";

pub(crate) struct Budget {
    started: Instant,
    limit: Duration,
    deadline_millis: Option<u64>,
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
        Ok(value) => {
            let limit = Duration::from_secs(seconds(&value)?);
            let deadline = match env::var(DEADLINE) {
                Ok(value) => Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| "Invalid private operator deadline")?,
                ),
                Err(env::VarError::NotPresent) => None,
                Err(_) => return Err("Private operator deadline is not valid Unicode".into()),
            };
            Ok(Some(Budget::with_deadline(limit, deadline)?))
        }
        Err(env::VarError::NotPresent) if env::var_os(DEADLINE).is_none() => Ok(None),
        Err(env::VarError::NotPresent) => {
            Err("Private deadline requires the existing configured timeout".into())
        }
        Err(_) => Err(format!("{VARIABLE} is not valid Unicode.")),
    }
}

pub(crate) fn check(budget: Option<&Budget>) -> Result<(), String> {
    budget.map_or(Ok(()), Budget::check)
}

pub(crate) fn child_command(command: &mut Command, budget: Option<&Budget>) -> Result<(), String> {
    if let Some(budget) = budget {
        let (seconds, deadline) = budget.child_allowance()?;
        command
            .env(VARIABLE, seconds.to_string())
            .env(DEADLINE, deadline.to_string());
    }
    Ok(())
}

impl Budget {
    pub(crate) fn new(limit: Duration) -> Self {
        let deadline_millis = SystemTime::now()
            .checked_add(limit)
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .and_then(|value| u64::try_from(value.as_millis()).ok());
        Self {
            started: Instant::now(),
            limit,
            deadline_millis,
        }
    }

    fn with_deadline(limit: Duration, deadline: Option<u64>) -> Result<Self, String> {
        let Some(deadline) = deadline else {
            return Ok(Self::new(limit));
        };
        let started = Instant::now();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?;
        let remaining = Duration::from_millis(deadline)
            .checked_sub(now)
            .unwrap_or(Duration::ZERO);
        let budget = Self {
            started,
            limit: limit.min(remaining),
            deadline_millis: Some(deadline),
        };
        budget.check()?;
        Ok(budget)
    }

    pub(crate) fn child_allowance(&self) -> Result<(u64, u64), String> {
        self.check()?;
        let seconds = self.limit.saturating_sub(self.started.elapsed()).as_secs();
        if seconds == 0 {
            return Err("Remaining operator budget cannot launch a nested preparation".into());
        }
        Ok((
            seconds,
            self.deadline_millis
                .ok_or("Operator deadline cannot be propagated")?,
        ))
    }

    pub(crate) fn check(&self) -> Result<(), String> {
        if self.started.elapsed() >= self.limit {
            Err("Release operator budget expired. Preserve the same source PR, immutable pin and run; retry the same supported operation after checking its retained state.".into())
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
    fn nested_operator_budget_never_restarts_or_extends_the_original_deadline() {
        let parent = Budget::new(Duration::from_secs(5));
        let (seconds, deadline) = parent.child_allowance().unwrap();
        assert!(seconds <= 5);
        let child = Budget::with_deadline(Duration::from_secs(28_800), Some(deadline)).unwrap();
        assert_eq!(child.deadline_millis, parent.deadline_millis);
        assert!(child.limit <= parent.limit);
        let mut command = Command::new("unused-fixture");
        child_command(&mut command, Some(&parent)).unwrap();
        let vars: Vec<_> = command.get_envs().collect();
        assert!(vars.iter().any(
            |(key, value)| *key == DEADLINE && value.unwrap() == deadline.to_string().as_str()
        ));
        assert!(Budget::with_deadline(Duration::from_secs(28_800), Some(1)).is_err());
        assert!(Budget::new(Duration::ZERO).child_allowance().is_err());
    }

    #[test]
    fn operator_polling_stops_at_the_remaining_budget() {
        let budget = Budget::new(Duration::from_millis(1));
        assert!(budget.sleep(Duration::from_secs(20)).is_err());
        assert!(budget.check().is_err());
    }
}
