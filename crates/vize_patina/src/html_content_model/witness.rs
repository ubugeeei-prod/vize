//! TS-36 for composed findings (Davinci P4-11b): every
//! `html/cross-component-nesting` error carries a witness chain the P4-6b
//! verifier re-checks against the fact base before it is reported.
//!
//! Two fact groups over a [`NestingEvidence`] artifact:
//!
//! ```text
//! stratum 0   HtmlElements          the authored element a proof cites
//! stratum 1   HtmlComposedNesting   the violation proven at a usage site
//! ```
//!
//! A finding's chain is the deciding ancestor element (when the class has
//! one), then the composed violation at the usage site. Spans are
//! skeleton-relative (template content offsets), as every fact of a group
//! is about one kind of span.

use vize_davinci::diagnostic::{Diagnostic, Stage, WitnessChain, WitnessLink};
use vize_davinci::fact::{
    Demand, FactConsumer, FactError, FactGroup, FactManager, FactProducer, FactRegistry, FactTable,
    FactView, ProducerEntry, ids,
};
use vize_davinci::pass::AnalysisId;
use vize_davinci::witness::{AuditReport, WitnessAudit, WitnessCheck, WitnessChecks, WitnessGroup};
use vize_s0::{Span, String};

use super::class::ViolationClass;
use super::composed::ComposedFinding;
use super::skeleton::{NodeKind, Skeleton};

/// An authored element a witness may cite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElementFact {
    /// Its start tag up to the end of its name.
    pub span: Span,
}

/// A violation proven at a component usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NestingFact {
    /// The usage site (the component's start tag) in the outermost file.
    pub span: Span,
    /// The violation class.
    pub class: ViolationClass,
}

/// What the groups are computed over: the cited elements and the findings,
/// both in finding order.
#[derive(Debug, Default)]
pub struct NestingEvidence {
    elements: Vec<ElementFact>,
    /// Per finding: the fact, and the index of its cited element.
    findings: Vec<(NestingFact, Option<u32>)>,
}

/// The span a diagnostic points at for `node`: an element's start tag up to
/// the end of its name, otherwise the node itself.
pub fn node_span(skeleton: &Skeleton, node: u32) -> Span {
    let entry = skeleton.node(node);
    match &entry.kind {
        NodeKind::Element(element) => Span::new(entry.span.start, element.name_span.end),
        NodeKind::Component { name } => Span::new(
            entry.span.start,
            entry.span.start + 1 + u32::try_from(name.len()).unwrap_or(0),
        ),
        _ => entry.span,
    }
}

impl NestingEvidence {
    /// The evidence for `findings` over `skeletons` (file `i` is
    /// `skeletons[i]`).
    pub fn new(skeletons: &[Skeleton], findings: &[ComposedFinding]) -> Self {
        let mut evidence = Self::default();
        for finding in findings {
            // `compose` always records the usage path; a bare finding cites
            // its own node so evidence stays aligned with `findings`.
            let (file, usage) = finding
                .usages
                .first()
                .copied()
                .unwrap_or((finding.file, finding.node));
            let cited = finding.evidence.and_then(|(file, node)| {
                let span = node_span(skeletons.get(file as usize)?, node);
                evidence.elements.push(ElementFact { span });
                Some(u32::try_from(evidence.elements.len() - 1).unwrap_or(u32::MAX))
            });
            let span = skeletons
                .get(file as usize)
                .map_or(Span::new(0, 0), |skeleton| node_span(skeleton, usage));
            let fact = NestingFact {
                span,
                class: finding.class,
            };
            evidence.findings.push((fact, cited));
        }
        evidence
    }
}

macro_rules! group {
    ($ty:ident, $id:expr, $name:literal, $stratum:expr, $depends:expr, $value:ty) => {
        /// A P4-11b fact group.
        pub struct $ty;
        impl FactGroup for $ty {
            const ID: AnalysisId = $id;
            const NAME: &'static str = $name;
            const STRATUM: u8 = $stratum;
            const DEPENDS: Demand = $depends;
            type Key = u32;
            type Value = $value;
        }
    };
}
group!(
    HtmlElements,
    ids::HTML_ELEMENTS,
    "html-elements",
    0,
    Demand::NONE,
    ElementFact
);
group!(
    HtmlComposedNesting,
    ids::HTML_COMPOSED_NESTING,
    "html-composed-nesting",
    1,
    Demand::NONE.with(ids::HTML_ELEMENTS),
    NestingFact
);

impl FactProducer<NestingEvidence> for HtmlElements {
    fn produce(evidence: &NestingEvidence, _: &FactView<'_>) -> FactTable<Self> {
        (0u32..).zip(evidence.elements.iter().copied()).collect()
    }
}

impl FactProducer<NestingEvidence> for HtmlComposedNesting {
    fn produce(evidence: &NestingEvidence, inputs: &FactView<'_>) -> FactTable<Self> {
        // A violation is a fact only when the element it cites is one.
        let elements = inputs.get::<HtmlElements>().ok();
        let cited = |index: &Option<u32>| {
            index.is_none_or(|index| elements.is_some_and(|table| table.get(&index).is_some()))
        };
        (0u32..)
            .zip(evidence.findings.iter())
            .filter(|(_, (_, index))| cited(index))
            .map(|(key, (fact, _))| (key, *fact))
            .collect()
    }
}

impl WitnessGroup for HtmlElements {
    fn fact_span(_: &u32, element: &ElementFact) -> Span {
        element.span
    }
}

impl WitnessGroup for HtmlComposedNesting {
    fn fact_span(_: &u32, nesting: &NestingFact) -> Span {
        nesting.span
    }
}

static REGISTRY: FactRegistry<NestingEvidence> = FactRegistry::new(&[
    ProducerEntry::of::<HtmlElements>(),
    ProducerEntry::of::<HtmlComposedNesting>(),
]);

/// The groups a `html/cross-component-nesting` witness may cite.
pub static CHECKS: WitnessChecks = WitnessChecks::new(&[
    WitnessCheck::of::<HtmlElements>(),
    WitnessCheck::of::<HtmlComposedNesting>(),
]);

/// The rule as a fact consumer.
pub struct CrossComponentNesting;

impl FactConsumer for CrossComponentNesting {
    const NAME: &'static str = "html/cross-component-nesting";
    const DEMAND: Demand = Demand::NONE
        .with(ids::HTML_ELEMENTS)
        .with(ids::HTML_COMPOSED_NESTING);
}

/// The facts of `evidence`, computed for [`CrossComponentNesting`].
///
/// # Errors
///
/// A [`FactError`] if the registry cannot compute the demand.
pub fn facts(
    evidence: &NestingEvidence,
) -> Result<FactManager<'static, NestingEvidence>, FactError> {
    let mut manager = FactManager::new(&REGISTRY);
    manager.compute(evidence, CrossComponentNesting::DEMAND)?;
    Ok(manager)
}

/// One proven diagnostic per finding, in finding order, each with its chain.
pub fn witnessed(evidence: &NestingEvidence, message: impl Fn(usize) -> String) -> Vec<Diagnostic> {
    (0u32..)
        .zip(evidence.findings.iter())
        .map(|(key, (fact, cited))| {
            let nesting = WitnessLink::of::<HtmlComposedNesting>(&key, fact.span);
            let element =
                cited.and_then(|index| Some((index, evidence.elements.get(index as usize)?)));
            let chain = match element {
                Some((index, element)) => {
                    WitnessChain::new(WitnessLink::of::<HtmlElements>(&index, element.span))
                        .then(nesting)
                }
                None => WitnessChain::new(nesting),
            };
            Diagnostic::proven(Stage::Semantic, fact.span, message(key as usize), chain)
        })
        .collect()
}

/// Audit every diagnostic's witness against the facts (TS-36): the report
/// of the debug/CI observer.
pub fn audit(evidence: &NestingEvidence, diagnostics: &[Diagnostic]) -> Option<AuditReport> {
    let manager = facts(evidence).ok()?;
    let mut audit = WitnessAudit::new();
    audit.audit(
        diagnostics,
        &manager.view::<CrossComponentNesting>(),
        &CHECKS,
    );
    Some(audit.report())
}
