//! TS-31 anchor classification and per-row aggregation.
//!
//! An authored anchor is
//! - `exact` when some segment maps a generated position whose bytes are the
//!   anchor's emitted form to the anchor's first authored byte;
//! - `covered` (but not exact) when some segment resolves inside the anchor's
//!   authored span without satisfying the exact rule;
//! - `unmapped` otherwise.
//!
//! Coverage is `(exact + covered) / authored`; span accuracy is
//! `exact / (exact + covered)`, so a map that lands near the right token but
//! not on it counts as covered yet inaccurate. An anchor a backend emits no
//! code for by design is `absent` once the harness has verified the absence,
//! and is not counted as authored for that backend.

use std::collections::BTreeMap;

use super::battery::{Anchor, Backend, Category, Emitted, is_identifier_char};
use super::decode::Segment;

/// Context rewrites a template identifier may carry in generated code.
const REWRITE_PREFIXES: [&str; 8] = [
    "",
    "_ctx.",
    "$setup.",
    "$props.",
    "$data.",
    "$options.",
    "__props.",
    "_unref(",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Exact,
    Covered,
    Unmapped,
    Absent,
}

impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Exact => "exact",
            Status::Covered => "covered",
            Status::Unmapped => "unmapped",
            Status::Absent => "absent",
        }
    }
}

/// Classify one authored anchor against a decoded map.
pub fn classify(
    anchor: &Anchor,
    backend: Backend,
    generated: &str,
    segments: &[Segment],
) -> Status {
    if anchor.absent.contains(&backend) {
        let emitted_anywhere = generated
            .char_indices()
            .any(|(offset, _)| emits(&generated[offset..], &anchor.emitted, backend));
        assert!(
            !emitted_anywhere,
            "{}: declared absent for {} but its emitted form is present",
            anchor.id,
            backend.as_str()
        );
        return Status::Absent;
    }
    let exact = segments.iter().any(|segment| {
        segment.source == Some(anchor.start)
            && segment
                .generated
                .is_some_and(|offset| emits(&generated[offset..], &anchor.emitted, backend))
    });
    if exact {
        return Status::Exact;
    }
    let span = anchor.start..anchor.start + anchor.len;
    let covered = segments
        .iter()
        .any(|segment| segment.source.is_some_and(|offset| span.contains(&offset)));
    if covered {
        Status::Covered
    } else {
        Status::Unmapped
    }
}

/// Whether `rest` (the generated text from a mapping onward) starts with the
/// anchor's emitted form.
fn emits(rest: &str, emitted: &Emitted, backend: Backend) -> bool {
    match emitted {
        Emitted::Verbatim(text) => rest.starts_with(text.as_str()),
        Emitted::Identifier(name) => REWRITE_PREFIXES.iter().any(|prefix| {
            rest.strip_prefix(prefix)
                .and_then(|tail| tail.strip_prefix(name.as_str()))
                .is_some_and(|after| !after.chars().next().is_some_and(is_identifier_char))
        }),
        Emitted::PerBackend { dom, vapor, ssr } => rest.starts_with(match backend {
            Backend::Dom => dom.as_str(),
            Backend::Vapor => vapor.as_str(),
            Backend::Ssr => ssr.as_str(),
            Backend::Sfc => panic!("section boundaries are template-backend anchors"),
        }),
    }
}

/// One `(backend, category)` measurement row.
#[derive(Debug, Default)]
pub struct Row {
    pub anchors: BTreeMap<std::string::String, Status>,
}

impl Row {
    fn count(&self, status: Status) -> usize {
        self.anchors.values().filter(|&&s| s == status).count()
    }

    pub fn to_json(&self) -> serde_json::Value {
        let exact = self.count(Status::Exact);
        let covered = exact + self.count(Status::Covered);
        let authored = self.anchors.len() - self.count(Status::Absent);
        let anchors = self
            .anchors
            .iter()
            .map(|(id, status)| (id.clone(), serde_json::json!(status.as_str())))
            .collect::<serde_json::Map<_, _>>();
        serde_json::json!({
            "authored": authored,
            "covered": covered,
            "exact": exact,
            "anchors": anchors,
        })
    }
}

/// Rows keyed `"<backend>/<category>"`, the report's row identity.
#[derive(Debug, Default)]
pub struct Rows(pub BTreeMap<std::string::String, Row>);

impl Rows {
    pub fn record(&mut self, backend: Backend, category: Category, id: &str, status: Status) {
        let key = std::format!("{}/{}", backend.as_str(), category.as_str());
        let previous = self
            .0
            .entry(key)
            .or_default()
            .anchors
            .insert(id.into(), status);
        assert_eq!(previous, None, "anchor `{id}` measured twice");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anchor(emitted: Emitted, start: usize, len: usize) -> Anchor {
        Anchor {
            id: "fixture:token@1:1".into(),
            category: Category::RewrittenIdentifier,
            start,
            len,
            emitted,
            absent: std::vec::Vec::new(),
        }
    }

    fn seg(generated: usize, source: usize) -> Segment {
        Segment {
            generated: Some(generated),
            source: Some(source),
        }
    }

    // Authored `{{ msg }}`: `msg` spans bytes 3..6. Generated `_ctx.msg;_ctx.msgs`.
    const GENERATED: &str = "_ctx.msg;_ctx.msgs";

    #[test]
    fn identifier_rules_require_start_byte_rewrite_and_word_boundary() {
        let msg = || anchor(Emitted::Identifier("msg".into()), 3, 3);
        let statuses = [
            classify(&msg(), Backend::Dom, GENERATED, &[seg(0, 3)]),
            classify(&msg(), Backend::Dom, GENERATED, &[seg(5, 3)]),
            classify(&msg(), Backend::Dom, GENERATED, &[seg(0, 4)]),
            classify(&msg(), Backend::Dom, GENERATED, &[seg(9, 3)]),
            classify(&msg(), Backend::Dom, GENERATED, &[seg(1, 3)]),
            classify(&msg(), Backend::Dom, GENERATED, &[seg(0, 6)]),
            classify(&msg(), Backend::Dom, GENERATED, &[]),
        ];
        assert_eq!(
            statuses,
            [
                Status::Exact,
                Status::Exact,
                Status::Covered,
                Status::Covered,
                Status::Covered,
                Status::Unmapped,
                Status::Unmapped,
            ]
        );
    }

    #[test]
    fn per_backend_constructs_are_selected_by_backend() {
        let boundary = anchor(
            Emitted::PerBackend {
                dom: "_ctx".into(),
                vapor: "msg".into(),
                ssr: ";".into(),
            },
            0,
            1,
        );
        let map = [seg(0, 0), seg(5, 0)];
        assert_eq!(
            [Backend::Dom, Backend::Vapor, Backend::Ssr]
                .map(|backend| classify(&boundary, backend, GENERATED, &map)),
            [Status::Exact, Status::Exact, Status::Covered]
        );
    }

    #[test]
    fn absent_anchors_are_verified_instead_of_measured() {
        let mut handler = anchor(Emitted::Identifier("inc".into()), 0, 3);
        handler.absent = std::vec![Backend::Ssr];
        assert_eq!(
            classify(&handler, Backend::Ssr, "_push(`<b>incr</b>`)", &[]),
            Status::Absent
        );
    }

    #[test]
    #[should_panic(expected = "declared absent for ssr but its emitted form is present")]
    fn absent_anchors_fail_when_the_backend_emits_the_token() {
        let mut handler = anchor(Emitted::Identifier("inc".into()), 0, 3);
        handler.absent = std::vec![Backend::Ssr];
        classify(&handler, Backend::Ssr, "onClick: _ctx.inc", &[]);
    }
}
