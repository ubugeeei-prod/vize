//! Own template complexity per component: the S2 `template-complexity`
//! facts (Davinci P4-9a), owned, with line/column positions so reports can
//! say where the complexity comes from without re-reading the file.

use vize_croquis::sfc::{SfcParseOptions, parse_sfc};
use vize_davinci::fact::{Demand, FactConsumer, FactGroup};
use vize_s1_to_s2::pass::cfg::{self, ComplexityFacts, DecisionKind, TemplateComplexityGroup};

/// The cross-file analyzer as a fact consumer: it reads the template
/// complexity group and nothing else (TS-35).
struct CrossFileTemplateFacts;

impl FactConsumer for CrossFileTemplateFacts {
    const NAME: &'static str = "croquis-cf/template-complexity";
    const DEMAND: Demand = Demand::NONE.with(TemplateComplexityGroup::ID);
}

/// Own cyclomatic complexity warns strictly above this (corpus p95).
pub const TEMPLATE_CYCLOMATIC_WARN_ABOVE: u32 = cfg::CYCLOMATIC_WARN_ABOVE;
/// Own cognitive complexity warns strictly above this (corpus p95).
pub const TEMPLATE_COGNITIVE_WARN_ABOVE: u32 = cfg::COGNITIVE_WARN_ABOVE;

/// Cyclomatic and cognitive complexity, own or rendered.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateScores {
    pub cyclomatic: u32,
    pub cognitive: u32,
}

impl TemplateScores {
    pub(crate) fn add(self, other: Self) -> Self {
        Self {
            cyclomatic: self.cyclomatic.saturating_add(other.cyclomatic),
            cognitive: self.cognitive.saturating_add(other.cognitive),
        }
    }
}

/// One construct that adds complexity, at its authored position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplexityContributor {
    /// The construct (`v-if`, `v-else-if`, `v-else`, `v-for`, `logical`,
    /// `conditional`), the metric spec's rule name.
    pub kind: &'static str,
    /// File byte offsets of the construct.
    pub start: u32,
    pub end: u32,
    /// One-based line and column (in characters) of `start`.
    pub line: u32,
    pub column: u32,
    /// Enclosing nesting regions.
    pub nesting: u32,
    pub cyclomatic: u32,
    pub cognitive: u32,
}

/// One component's own template complexity.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateComplexity {
    pub own: TemplateScores,
    /// Evaluated positions without a retained expression AST.
    pub unknown: u32,
    pub max_nesting: u32,
    pub scoped_slots: u32,
    /// Every construct with a non-zero increment, in source order.
    pub contributors: Vec<ComplexityContributor>,
}

impl TemplateComplexity {
    /// The facts of an SFC's template block, `None` without an HTML
    /// template (no template, `lang="pug"`, an external `src`).
    pub fn from_sfc(source: &str) -> Option<Self> {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
        let template = descriptor.template.as_ref()?;
        if template.src.is_some() || template.lang.as_deref().is_some_and(|lang| lang != "html") {
            return None;
        }
        let start = u32::try_from(template.loc.start).ok()?;
        let end = u32::try_from(template.loc.end).ok()?;
        let facts = cfg::template_facts::<CrossFileTemplateFacts>(source, start, end)?;
        Some(Self::from_facts(&facts, source))
    }

    /// Own the pass product, resolving each row's position in `source`.
    pub fn from_facts(facts: &ComplexityFacts, source: &str) -> Self {
        let mut lines = LineCursor::new(source);
        let contributors = facts
            .contributions
            .iter()
            .filter(|row| row.cyclomatic > 0 || row.cognitive > 0)
            .map(|row| {
                let (line, column) = lines.position(row.span.start);
                ComplexityContributor {
                    kind: row.kind.as_str(),
                    start: row.span.start,
                    end: row.span.end,
                    line,
                    column,
                    nesting: row.nesting,
                    cyclomatic: row.cyclomatic,
                    cognitive: row.cognitive,
                }
            })
            .collect();
        let scoped_slots = facts
            .contributions
            .iter()
            .filter(|row| row.kind == DecisionKind::ScopedSlot)
            .count();
        Self {
            own: TemplateScores {
                cyclomatic: facts.cyclomatic,
                cognitive: facts.cognitive,
            },
            unknown: facts.unknown,
            max_nesting: facts.max_nesting,
            scoped_slots: u32::try_from(scoped_slots).unwrap_or(u32::MAX),
            contributors,
        }
    }

    /// Whether the own scores exceed the default thresholds.
    pub const fn exceeds_thresholds(&self) -> bool {
        self.own.cyclomatic > TEMPLATE_CYCLOMATIC_WARN_ABOVE
            || self.own.cognitive > TEMPLATE_COGNITIVE_WARN_ABOVE
    }

    /// The `limit` largest contributors (cognitive increment first, then
    /// cyclomatic, then source order), returned in source order.
    pub fn top_contributors(&self, limit: usize) -> Vec<&ComplexityContributor> {
        let mut ranked: Vec<&ComplexityContributor> = self.contributors.iter().collect();
        ranked.sort_by(|left, right| {
            right
                .cognitive
                .cmp(&left.cognitive)
                .then(right.cyclomatic.cmp(&left.cyclomatic))
                .then(left.start.cmp(&right.start))
        });
        ranked.truncate(limit);
        ranked.sort_by_key(|row| (row.start, row.end));
        ranked
    }
}

/// Forward-only offset → (line, column) resolution; rows arrive in source
/// order, so the whole breakdown costs one pass over the file.
struct LineCursor<'a> {
    source: &'a str,
    offset: usize,
    line: u32,
    line_start: usize,
}

impl<'a> LineCursor<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            offset: 0,
            line: 1,
            line_start: 0,
        }
    }

    fn position(&mut self, target: u32) -> (u32, u32) {
        let target = (target as usize).min(self.source.len());
        if target < self.offset {
            *self = Self::new(self.source);
        }
        let bytes = self.source.as_bytes();
        while self.offset < target {
            if bytes[self.offset] == b'\n' {
                self.line = self.line.saturating_add(1);
                self.line_start = self.offset + 1;
            }
            self.offset += 1;
        }
        let column = self
            .source
            .get(self.line_start..target)
            .map_or(0, |prefix| prefix.chars().count());
        (self.line, u32::try_from(column + 1).unwrap_or(u32::MAX))
    }
}
