use alloc::vec::Vec;
use core::fmt;
use core::str::SplitWhitespace;

use vize_davinci::folio::value::FolioValue;
use vize_davinci::folio::{Folio, FolioError};
use vize_s0::{Span, cstr};

use super::{
    BindingFact, BindingId, BindingOrigin, EffectSet, EscapeKind, LatticeFacts, ReactivityClass,
    Verdict,
};

/// Owned folio page for the S3 reactivity lattice.
#[derive(Debug, Clone, Default, PartialEq, Eq, Folio)]
pub struct S3ReactivityFolio {
    /// Binding facts in binding-id order.
    pub bindings: Vec<FolioBinding>,
}

/// Compatibility alias for the shorter feature name.
pub type ReactivityFolio = S3ReactivityFolio;

impl S3ReactivityFolio {
    /// Mirror live arena facts into the owned folio page.
    #[must_use]
    pub fn of(facts: &LatticeFacts<'_>) -> Self {
        Self {
            bindings: facts.bindings.iter().map(FolioBinding::from).collect(),
        }
    }
}

/// One lattice fact line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FolioBinding {
    pub id: u32,
    pub class: ReactivityClass,
    pub verdict: Verdict,
    pub origin: BindingOrigin,
    pub effects: EffectSet,
    pub escape: EscapeKind,
    pub span: Span,
}

impl From<&BindingFact> for FolioBinding {
    fn from(fact: &BindingFact) -> Self {
        Self {
            id: fact.id.index(),
            class: fact.class,
            verdict: fact.verdict,
            origin: fact.origin,
            effects: fact.effects,
            escape: fact.escape,
            span: fact.span,
        }
    }
}

impl FolioValue for FolioBinding {
    fn print_value<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        write!(
            w,
            "id={} class={} verdict={} origin={} effects=",
            self.id,
            self.class.as_str(),
            self.verdict.as_str(),
            self.origin.as_str()
        )?;
        self.effects.print(w)?;
        write!(
            w,
            " escape={} span={}:{}",
            self.escape.as_str(),
            self.span.start,
            self.span.end
        )
    }

    fn parse_value(text: &str, line: usize) -> Result<Self, FolioError> {
        let mut fields = text.split_whitespace();
        let id = parse_u32(next(&mut fields, "id", line)?, "id", line)?;
        let class = parse_class(next(&mut fields, "class", line)?, line)?;
        let verdict = parse_verdict(next(&mut fields, "verdict", line)?, line)?;
        let origin = parse_origin(next(&mut fields, "origin", line)?, line)?;
        let effects = parse_effects(next(&mut fields, "effects", line)?, line)?;
        let escape = parse_escape(next(&mut fields, "escape", line)?, line)?;
        let span = parse_span(next(&mut fields, "span", line)?, line)?;
        expect_end(fields, line)?;
        Ok(Self {
            id,
            class,
            verdict,
            origin,
            effects,
            escape,
            span,
        })
    }
}

impl From<FolioBinding> for BindingFact {
    fn from(binding: FolioBinding) -> Self {
        Self {
            id: BindingId::new(binding.id),
            class: binding.class,
            verdict: binding.verdict,
            origin: binding.origin,
            effects: binding.effects,
            escape: binding.escape,
            span: binding.span,
        }
    }
}

fn next<'a>(
    fields: &mut SplitWhitespace<'a>,
    name: &str,
    line: usize,
) -> Result<&'a str, FolioError> {
    let Some(raw) = fields.next() else {
        return Err(FolioError::new(line, cstr!("missing `{name}` field")));
    };
    raw.strip_prefix(name)
        .and_then(|rest| rest.strip_prefix('='))
        .ok_or_else(|| FolioError::new(line, cstr!("expected `{name}=...`, got `{raw}`")))
}

fn parse_u32(raw: &str, name: &str, line: usize) -> Result<u32, FolioError> {
    raw.parse()
        .map_err(|_| FolioError::new(line, cstr!("invalid `{name}` value `{raw}`")))
}

fn parse_class(raw: &str, line: usize) -> Result<ReactivityClass, FolioError> {
    ReactivityClass::from_str(raw)
        .ok_or_else(|| FolioError::new(line, cstr!("unknown reactivity class `{raw}`")))
}

fn parse_verdict(raw: &str, line: usize) -> Result<Verdict, FolioError> {
    Verdict::from_str(raw).ok_or_else(|| FolioError::new(line, cstr!("unknown verdict `{raw}`")))
}

fn parse_origin(raw: &str, line: usize) -> Result<BindingOrigin, FolioError> {
    BindingOrigin::from_str(raw)
        .ok_or_else(|| FolioError::new(line, cstr!("unknown binding origin `{raw}`")))
}

fn parse_effects(raw: &str, line: usize) -> Result<EffectSet, FolioError> {
    EffectSet::parse(raw).ok_or_else(|| FolioError::new(line, cstr!("unknown effect set `{raw}`")))
}

fn parse_escape(raw: &str, line: usize) -> Result<EscapeKind, FolioError> {
    EscapeKind::from_str(raw)
        .ok_or_else(|| FolioError::new(line, cstr!("unknown escape kind `{raw}`")))
}

fn parse_span(raw: &str, line: usize) -> Result<Span, FolioError> {
    let Some((start, end)) = raw.split_once(':') else {
        return Err(FolioError::new(line, cstr!("invalid span `{raw}`")));
    };
    Ok(Span::new(
        parse_u32(start, "span start", line)?,
        parse_u32(end, "span end", line)?,
    ))
}

fn expect_end(mut fields: SplitWhitespace<'_>, line: usize) -> Result<(), FolioError> {
    match fields.next() {
        Some(extra) => Err(FolioError::new(line, cstr!("unexpected field `{extra}`"))),
        None => Ok(()),
    }
}
