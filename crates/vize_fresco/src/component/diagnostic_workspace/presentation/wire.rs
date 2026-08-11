use compact_str::CompactString;
use serde::{Deserialize, Deserializer, de};

use super::{
    DiagnosticPresentation, DiagnosticPresentationError, DiagnosticPresentationKind,
    DiagnosticTone, visible_text,
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DiagnosticPresentationWire {
    kind: DiagnosticPresentationKind,
    tone: DiagnosticTone,
    value: CompactString,
    #[serde(default)]
    description: Option<CompactString>,
    #[serde(default)]
    score: Option<(u64, u64)>,
    #[serde(default)]
    set_position: Option<(u64, u64)>,
}

impl<'de> Deserialize<'de> for DiagnosticPresentation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        DiagnosticPresentationWire::deserialize(deserializer)
            .and_then(|wire| wire.try_into().map_err(de::Error::custom))
    }
}

impl TryFrom<DiagnosticPresentationWire> for DiagnosticPresentation {
    type Error = DiagnosticPresentationError;

    fn try_from(wire: DiagnosticPresentationWire) -> Result<Self, Self::Error> {
        let canonical_value = visible_text(wire.value.clone(), "value")?;
        let mut presentation = match wire.kind {
            DiagnosticPresentationKind::Score => score(&wire, &canonical_value)?,
            DiagnosticPresentationKind::CodeLocation => location(&wire, &canonical_value)?,
            DiagnosticPresentationKind::Evidence => evidence(&wire, canonical_value)?,
            DiagnosticPresentationKind::KeyHint => key_hint(&wire, &canonical_value)?,
            kind => {
                require_no_metadata(&wire, kind)?;
                DiagnosticPresentation::new(kind, canonical_value, wire.tone)?
            }
        };
        if let Some(description) = wire.description {
            presentation = presentation.with_description(description)?;
        }
        Ok(presentation)
    }
}

fn score(
    wire: &DiagnosticPresentationWire,
    canonical_value: &str,
) -> Result<DiagnosticPresentation, DiagnosticPresentationError> {
    if wire.set_position.is_some() {
        return invalid(
            wire.kind,
            "score presentations cannot have set position metadata",
        );
    }
    let (value, maximum) = wire
        .score
        .ok_or(DiagnosticPresentationError::InvalidStructure {
            kind: wire.kind,
            reason: "score presentations require score metadata",
        })?;
    let presentation = DiagnosticPresentation::score(value, maximum, wire.tone)?;
    require_canonical_value(wire.kind, canonical_value, presentation.value())?;
    Ok(presentation)
}

fn location(
    wire: &DiagnosticPresentationWire,
    canonical_value: &str,
) -> Result<DiagnosticPresentation, DiagnosticPresentationError> {
    require_no_metadata(wire, wire.kind)?;
    let (path_and_line, column) = canonical_value
        .rsplit_once(':')
        .ok_or(DiagnosticPresentationError::InvalidCodeLocation)?;
    let (path, line) = path_and_line
        .rsplit_once(':')
        .ok_or(DiagnosticPresentationError::InvalidCodeLocation)?;
    let line = line
        .parse()
        .map_err(|_| DiagnosticPresentationError::InvalidCodeLocation)?;
    let column = column
        .parse()
        .map_err(|_| DiagnosticPresentationError::InvalidCodeLocation)?;
    let presentation = DiagnosticPresentation::code_location(path, line, column)?;
    require_canonical_value(wire.kind, canonical_value, presentation.value())?;
    Ok(presentation)
}

fn evidence(
    wire: &DiagnosticPresentationWire,
    canonical_value: CompactString,
) -> Result<DiagnosticPresentation, DiagnosticPresentationError> {
    if wire.score.is_some() {
        return invalid(
            wire.kind,
            "evidence presentations cannot have score metadata",
        );
    }
    if wire.tone != DiagnosticTone::Informational {
        return invalid(
            wire.kind,
            "evidence presentations use the informational tone",
        );
    }
    let (position, set_size) =
        wire.set_position
            .ok_or(DiagnosticPresentationError::InvalidStructure {
                kind: wire.kind,
                reason: "evidence presentations require set position metadata",
            })?;
    DiagnosticPresentation::evidence(canonical_value, position, set_size)
}

fn key_hint(
    wire: &DiagnosticPresentationWire,
    canonical_value: &str,
) -> Result<DiagnosticPresentation, DiagnosticPresentationError> {
    require_no_metadata(wire, wire.kind)?;
    if wire.tone != DiagnosticTone::Neutral {
        return invalid(wire.kind, "key hints use the neutral tone");
    }
    let (key, action) =
        canonical_value
            .split_once(": ")
            .ok_or(DiagnosticPresentationError::InvalidStructure {
                kind: wire.kind,
                reason: "key hints require the canonical `key: action` value",
            })?;
    let presentation = DiagnosticPresentation::key_hint(key, action)?;
    require_canonical_value(wire.kind, canonical_value, presentation.value())?;
    Ok(presentation)
}

fn require_no_metadata(
    wire: &DiagnosticPresentationWire,
    kind: DiagnosticPresentationKind,
) -> Result<(), DiagnosticPresentationError> {
    if wire.score.is_some() || wire.set_position.is_some() {
        return invalid(
            kind,
            "this presentation kind does not accept structured metadata",
        );
    }
    Ok(())
}

fn require_canonical_value(
    kind: DiagnosticPresentationKind,
    supplied: &str,
    canonical: &str,
) -> Result<(), DiagnosticPresentationError> {
    if supplied != canonical {
        return invalid(
            kind,
            "displayed value disagrees with its structured metadata",
        );
    }
    Ok(())
}

fn invalid<T>(
    kind: DiagnosticPresentationKind,
    reason: &'static str,
) -> Result<T, DiagnosticPresentationError> {
    Err(DiagnosticPresentationError::InvalidStructure { kind, reason })
}
