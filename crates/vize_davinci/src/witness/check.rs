//! [`WitnessGroup`] — a fact group a witness may cite — and the
//! [`WitnessChecks`] registry the verifier dispatches through.

use super::WitnessError;
use crate::diagnostic::{Verdict, WitnessKeyed, WitnessLink};
use crate::fact::{Demand, FactGroup, FactView};
use crate::pass::AnalysisId;
use vize_s0::Span;

/// A fact group a witness link may cite: its key erases to a
/// [`WitnessKey`](crate::diagnostic::WitnessKey), and every fact states the
/// span it is about and its verdict.
///
/// The span is what makes a link checkable rather than merely nameable: a
/// witness that points at the right fact from the wrong place is a forged
/// witness.
pub trait WitnessGroup: FactGroup<Key: WitnessKeyed> {
    /// The authored-file range the fact stored under `key` is about.
    fn fact_span(key: &Self::Key, value: &Self::Value) -> Span;

    /// The fact's verdict. A structural fact is proven by construction; a
    /// three-valued fact overrides this, and only a proven fact backs a
    /// witness.
    fn verdict(_value: &Self::Value) -> Verdict {
        Verdict::Proven
    }
}

/// How the verifier re-checks one link against one group's table.
type CheckFn = fn(&FactView<'_>, usize, &WitnessLink) -> Result<(), WitnessError>;

/// One witness-capable group, as const registry data: its identity and the
/// monomorphized check. Dispatch is one function pointer per link.
#[derive(Clone, Copy)]
pub struct WitnessCheck {
    /// The group this entry checks.
    pub group: AnalysisId,
    /// The group's stable name.
    pub name: &'static str,
    check: CheckFn,
}

impl core::fmt::Debug for WitnessCheck {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WitnessCheck")
            .field("group", &self.group)
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

impl WitnessCheck {
    /// The entry for group `G`.
    #[must_use]
    pub const fn of<G: WitnessGroup>() -> Self {
        Self {
            group: G::ID,
            name: G::NAME,
            check: check_link::<G>,
        }
    }

    /// Re-check `link` (the `index`-th of its chain) against `facts`.
    ///
    /// # Errors
    ///
    /// The exact [`WitnessError`] for the first check that fails.
    pub fn run(
        &self,
        facts: &FactView<'_>,
        index: usize,
        link: &WitnessLink,
    ) -> Result<(), WitnessError> {
        (self.check)(facts, index, link)
    }
}

fn check_link<G: WitnessGroup>(
    facts: &FactView<'_>,
    index: usize,
    link: &WitnessLink,
) -> Result<(), WitnessError> {
    let table = facts
        .get::<G>()
        .map_err(|error| WitnessError::Fact { link: index, error })?;
    let Some(key) = G::Key::from_witness_key(&link.key) else {
        return Err(WitnessError::KeyShape {
            link: index,
            group: G::ID,
            key: link.key.clone(),
        });
    };
    let Some(value) = table.get(&key) else {
        return Err(WitnessError::MissingKey {
            link: index,
            group: G::ID,
            key: link.key.clone(),
        });
    };
    let fact = G::fact_span(&key, value);
    if fact != link.span {
        return Err(WitnessError::SpanMismatch {
            link: index,
            group: G::ID,
            fact,
            witness: link.span,
        });
    }
    let verdict = G::verdict(value);
    if !verdict.is_proven() {
        return Err(WitnessError::NotProven {
            link: index,
            group: G::ID,
            verdict,
        });
    }
    Ok(())
}

/// The witness-capable groups, as const data.
///
/// A group appears at most once; [`WitnessChecks::new`] rejects a repeat in
/// const evaluation, so a registry that registers a group twice does not
/// build:
///
/// ```compile_fail,E0080
/// use vize_davinci::diagnostic::Verdict;
/// use vize_davinci::fact::{Demand, FactGroup};
/// use vize_davinci::pass::AnalysisId;
/// use vize_davinci::witness::{WitnessCheck, WitnessChecks, WitnessGroup};
/// use vize_s0::Span;
///
/// struct Tags;
/// impl FactGroup for Tags {
///     const ID: AnalysisId = AnalysisId::new(4);
///     const NAME: &'static str = "tags";
///     const STRATUM: u8 = 0;
///     const DEPENDS: Demand = Demand::NONE;
///     type Key = u32;
///     type Value = Span;
/// }
/// impl WitnessGroup for Tags {
///     fn fact_span(_: &u32, span: &Span) -> Span {
///         *span
///     }
/// }
///
/// static CHECKS: WitnessChecks =
///     WitnessChecks::new(&[WitnessCheck::of::<Tags>(), WitnessCheck::of::<Tags>()]);
/// ```
///
/// Its twin, registering the group once, builds — which pins the failure to
/// the repeat (stable rustdoc does not check the error code):
///
/// ```
/// use vize_davinci::diagnostic::Verdict;
/// use vize_davinci::fact::{Demand, FactGroup};
/// use vize_davinci::pass::AnalysisId;
/// use vize_davinci::witness::{WitnessCheck, WitnessChecks, WitnessGroup};
/// use vize_s0::Span;
///
/// struct Tags;
/// impl FactGroup for Tags {
///     const ID: AnalysisId = AnalysisId::new(4);
///     const NAME: &'static str = "tags";
///     const STRATUM: u8 = 0;
///     const DEPENDS: Demand = Demand::NONE;
///     type Key = u32;
///     type Value = Span;
/// }
/// impl WitnessGroup for Tags {
///     fn fact_span(_: &u32, span: &Span) -> Span {
///         *span
///     }
/// }
///
/// static CHECKS: WitnessChecks = WitnessChecks::new(&[WitnessCheck::of::<Tags>()]);
/// assert_eq!(CHECKS.groups(), Demand::NONE.with(Tags::ID));
/// assert_eq!(Tags::verdict(&Span::new(0, 1)), Verdict::Proven);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct WitnessChecks {
    checks: &'static [WitnessCheck],
    groups: Demand,
}

impl WitnessChecks {
    /// The registry of `checks`.
    ///
    /// # Panics
    ///
    /// Panics if a group is registered twice. In a `const` or `static` item
    /// — how registries are declared — that panic is a compile error.
    #[must_use]
    pub const fn new(checks: &'static [WitnessCheck]) -> Self {
        let mut groups = Demand::NONE;
        let mut index = 0;
        while index < checks.len() {
            let group = checks[index].group;
            assert!(
                !groups.contains(group),
                "a witness registry names each fact group at most once"
            );
            groups = groups.with(group);
            index += 1;
        }
        Self { checks, groups }
    }

    /// The groups a witness may cite.
    #[must_use]
    pub const fn groups(&self) -> Demand {
        self.groups
    }

    /// The check registered for `group`.
    #[must_use]
    pub fn check(&self, group: AnalysisId) -> Option<&'static WitnessCheck> {
        if !self.groups.contains(group) {
            return None;
        }
        self.checks.iter().find(|check| check.group == group)
    }
}
