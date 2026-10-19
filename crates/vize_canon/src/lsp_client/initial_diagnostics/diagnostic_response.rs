//! Wire-to-pull-diagnostic conversion for native 7.0.2, without an AST/disk view.
//!
//! Primary contract: microsoft/typescript-go@2bd066d87f5bafd315be9f40889d0a60b9e58e0b
//! internal/api/proto.go, internal/ls/lsconv/{converters,linemap}.go and
//! internal/diagnosticwriter/diagnosticwriter.go (MIT, Copyright Microsoft).
//! This mirrors Vize's private PULL client, not the different push converter.

use corsa::CorsaError;
use std::{path::Path, str::FromStr};

use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString,
    Position, Range, Uri,
};
use serde::Deserialize;
use vize_l0::{String, cstr};

use crate::file_uri::path_to_file_uri;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NativeDiagnostic {
    #[serde(default)]
    file_name: Option<String>,
    pos: i64,
    end: i64,
    code: i32,
    category: u8,
    text: String,
    #[serde(default)]
    message_chain: Vec<Self>,
    #[serde(default)]
    related_information: Vec<Self>,
    // The current private pull client does not advertise diagnostic tagValueSet.
    // Deserialize these known fields, but do not invent tags from them.
    #[serde(default, rename = "reportsUnnecessary")]
    _reports_unnecessary: bool,
    #[serde(default, rename = "reportsDeprecated")]
    _reports_deprecated: bool,
}

impl NativeDiagnostic {
    pub(super) fn belongs_to(&self, expected_file: &str) -> bool {
        self.file_name
            .as_deref()
            .is_none_or(|file| file == expected_file)
    }

    pub(super) fn source_names(&self) -> impl Iterator<Item = &str> {
        self.file_name.as_deref().into_iter().chain(
            self.related_information
                .iter()
                .filter_map(|related| related.file_name.as_deref()),
        )
    }

    pub(super) fn into_lsp<'a>(
        self,
        expected_file: &str,
        mut text: impl FnMut(&str) -> Result<&'a SnapshotText, String>,
    ) -> Result<Diagnostic, String> {
        if !self.belongs_to(expected_file) {
            return Err(cstr!("Native file diagnostic belongs to another source"));
        }
        let range = self.range(&mut text)?;
        let mut related_information = Vec::with_capacity(self.related_information.len());
        for related in &self.related_information {
            let file = related
                .file_name
                .as_deref()
                .ok_or_else(|| cstr!("Native related diagnostic has no source"))?;
            if !Path::new(file).is_absolute() || file.contains('\0') {
                return Err(cstr!(
                    "Native related diagnostic has invalid source identity"
                ));
            }
            let uri = Uri::from_str(path_to_file_uri(Path::new(file)).as_str())
                .map_err(|error| cstr!("Invalid native related source URI: {error}"))?;
            related_information.push(DiagnosticRelatedInformation {
                location: Location::new(uri, related.range(&mut text)?),
                // Native pull converts related.Localize(), not its message chain.
                message: related.text.as_str().into(),
            });
        }
        let severity = match self.category {
            0 => DiagnosticSeverity::WARNING,
            1 if matches!(
                self.code,
                6133 | 6138 | 6192 | 6196 | 7027 | 7028 | 7029 | 7030
            ) =>
            {
                // NewDefaultUserPreferences=true; Vize does not override it.
                DiagnosticSeverity::WARNING
            }
            1 => DiagnosticSeverity::ERROR,
            2 => DiagnosticSeverity::HINT,
            3 => DiagnosticSeverity::INFORMATION,
            _ => return Err(cstr!("Invalid native diagnostic category")),
        };
        let mut message = self.text.clone();
        self.append_chain(&mut message, 1);
        Ok(Diagnostic {
            range,
            severity: Some(severity),
            code: Some(NumberOrString::Number(self.code)),
            code_description: None,
            source: Some("ts".into()),
            message: message.as_str().into(),
            related_information: (!related_information.is_empty()).then_some(related_information),
            tags: None,
            data: None,
        })
    }

    fn append_chain(&self, message: &mut String, depth: usize) {
        for child in &self.message_chain {
            message.push('\n');
            for _ in 0..depth {
                message.push_str("  ");
            }
            message.push_str(&child.text);
            child.append_chain(message, depth + 1);
        }
    }

    fn range<'a>(
        &self,
        text: &mut impl FnMut(&str) -> Result<&'a SnapshotText, String>,
    ) -> Result<Range, String> {
        let Some(file) = &self.file_name else {
            return Ok(Range::default());
        };
        if self.pos < 0 || self.end < self.pos {
            return Err(cstr!("Invalid native diagnostic span"));
        }
        let text = text(file)?;
        Ok(Range::new(
            text.position(self.pos)?,
            text.position(self.end)?,
        ))
    }
}

/// One immutable snapshot source and its UTF16 line index, scoped to ONE read.
/// This is not a filesystem stamp/result cache and cannot escape that read.
pub(super) struct SnapshotText {
    _text: String,
    line_starts: Vec<u32>,
    len: u32,
}

impl SnapshotText {
    pub(super) fn new(text: &str) -> Result<Self, String> {
        let mut units = text.encode_utf16().peekable();
        let mut len = 0_u32;
        let mut line_starts = vec![0];
        while let Some(unit) = units.next() {
            len = len
                .checked_add(1)
                .ok_or_else(|| cstr!("Native UTF16 source exceeds u32"))?;
            if unit == 13 && units.peek() == Some(&10) {
                units.next();
                len = len
                    .checked_add(1)
                    .ok_or_else(|| cstr!("Native UTF16 source exceeds u32"))?;
            }
            // Native lsconv.ComputeLSPLineStarts uses CR/LF/CRLF only.
            // ECMAScript LS/PS remain ordinary UTF16 columns in LSP.
            if matches!(unit, 10 | 13) {
                line_starts.push(len);
            }
        }
        Ok(Self {
            _text: text.into(),
            line_starts,
            len,
        })
    }

    fn position(&self, offset: i64) -> Result<Position, String> {
        let offset = u32::try_from(offset).map_err(|_| cstr!("Invalid native UTF16 offset"))?;
        if offset > self.len {
            return Err(cstr!("Native UTF16 span exceeds its snapshot source"));
        }
        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .checked_sub(1)
            .ok_or_else(|| cstr!("Invalid native line index"))?;
        let start = self
            .line_starts
            .get(line)
            .ok_or_else(|| cstr!("Invalid native line index"))?;
        Ok(Position::new(
            u32::try_from(line).map_err(|_| cstr!("Invalid native line index"))?,
            offset - *start,
        ))
    }
}

// Only native's exact internal-error envelope or standard method-not-found may
// authorize fallback. Identical text under client-argument/transport codes fails.
pub(super) fn missing_method(method: &str, error: &CorsaError) -> bool {
    let missing = |message: &str| {
        message == cstr!("unknown API method \"{method}\"").as_str()
            || message == cstr!("invalid request: unknown API method \"{method}\"").as_str()
    };
    match error {
        CorsaError::Rpc(rpc) => rpc.code == -32601 || (rpc.code == -32603 && missing(&rpc.message)),
        CorsaError::Protocol(message) => missing(message),
        CorsaError::Unsupported(_) => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{CorsaError, NativeDiagnostic, Position, SnapshotText, missing_method};
    use serde_json::json;

    fn wire(code: i32, category: u8) -> serde_json::Value {
        json!({"fileName":"/workspace/a.ts","pos":5,"end":7,"code":code,
            "category":category,"text":"whole root","reportsUnnecessary":true,
            "reportsDeprecated":true})
    }

    #[test]
    fn unicode_crlf_chains_related_text_and_whole_pull_packet_are_preserved() {
        let mut value = wire(2322, 1);
        value["messageChain"] = json!([
            {"pos":0,"end":0,"code":2322,"category":1,"text":"first",
             "messageChain":[{"pos":0,"end":0,"code":2322,"category":1,"text":"nested"}]},
            {"pos":0,"end":0,"code":2322,"category":1,"text":"second"}]);
        value["relatedInformation"] = json!([{"fileName":"/workspace/b.ts",
            "pos":2,"end":3,"code":1,"category":3,"text":"related authored",
            "messageChain":[{"pos":0,"end":0,"code":1,"category":3,"text":"not flattened"}]}]);
        let primary = SnapshotText::new("😀a\r\nbc\u{2028}d\re\nf").unwrap();
        let related = SnapshotText::new("😀z").unwrap();
        let packet = serde_json::from_value::<NativeDiagnostic>(value)
            .unwrap()
            .into_lsp("/workspace/a.ts", |name| {
                Ok(if name.ends_with("a.ts") {
                    &primary
                } else {
                    &related
                })
            })
            .unwrap();
        assert_eq!(
            serde_json::to_value(packet).unwrap(),
            json!({
            "range":{"start":{"line":1,"character":0},"end":{"line":1,"character":2}},
            "severity":1,"code":2322,"source":"ts",
            "message":"whole root\n  first\n    nested\n  second",
            "relatedInformation":[{"location":{"uri":"file:///workspace/b.ts",
                "range":{"start":{"line":0,"character":2},"end":{"line":0,"character":3}}},
                "message":"related authored"}]})
        );
    }

    #[test]
    fn pull_style_codes_and_all_categories_keep_exact_severities_without_tags() {
        let source = SnapshotText::new("1234567").unwrap();
        for (code, category, severity) in [
            (6133, 1, 2),
            (6138, 1, 2),
            (6192, 1, 2),
            (6196, 1, 2),
            (7027, 1, 2),
            (7028, 1, 2),
            (7029, 1, 2),
            (7030, 1, 2),
            (2322, 1, 1),
            (6133, 0, 2),
            (6133, 2, 4),
            (6133, 3, 3),
        ] {
            let packet = serde_json::from_value::<NativeDiagnostic>(wire(code, category))
                .unwrap()
                .into_lsp("/workspace/a.ts", |_| Ok(&source))
                .unwrap();
            assert_eq!(
                serde_json::to_value(packet).unwrap(),
                json!({
                "range":{"start":{"line":0,"character":5},"end":{"line":0,"character":7}},
                "severity":severity,"code":code,"source":"ts","message":"whole root"})
            );
        }
    }

    #[test]
    fn foreign_source_missing_related_text_and_invalid_spans_refuse_whole_packet() {
        let source = SnapshotText::new("1234567").unwrap();
        for (field, value) in [
            ("pos", json!(-1)),
            ("end", json!(4)),
            ("end", json!(8)),
            ("category", json!(4)),
            ("fileName", json!("/foreign/a.ts")),
        ] {
            let mut row = wire(2322, 1);
            row[field] = value;
            assert!(
                serde_json::from_value::<NativeDiagnostic>(row)
                    .unwrap()
                    .into_lsp("/workspace/a.ts", |_| Ok(&source))
                    .is_err()
            );
        }
        let mut row = wire(2322, 1);
        row["relatedInformation"] = json!([{"fileName":"/missing.ts","pos":0,"end":0,
            "code":1,"category":3,"text":"must not drop"}]);
        assert!(
            serde_json::from_value::<NativeDiagnostic>(row)
                .unwrap()
                .into_lsp("/workspace/a.ts", |name| if name == "/missing.ts" {
                    Err("missing snapshot text".into())
                } else {
                    Ok(&source)
                })
                .is_err()
        );
        for field in ["pos", "end", "code", "category", "text"] {
            let mut row = wire(2322, 1);
            row.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<NativeDiagnostic>(row).is_err());
        }
    }

    #[test]
    fn absolute_utf16_line_positions_do_not_clamp_or_normalize_source() {
        let text = SnapshotText::new("😀a\r\nbc\u{2028}d\re\nf").unwrap();
        for (offset, line, column) in [
            (0, 0, 0),
            (1, 0, 1),
            (2, 0, 2),
            (4, 0, 4),
            (5, 1, 0),
            (7, 1, 2),
            (8, 1, 3),
            (10, 2, 0),
            (12, 3, 0),
            (13, 3, 1),
        ] {
            assert_eq!(text.position(offset).unwrap(), Position::new(line, column));
        }
        assert!(text.position(14).is_err());
        assert!(text.position(-1).is_err());
        let separators = SnapshotText::new("a\u{2028}😀\u{2029}z").unwrap();
        for offset in 0..=6 {
            assert_eq!(
                separators.position(offset).unwrap(),
                Position::new(0, offset as u32)
            );
        }
        assert!(separators.position(7).is_err());
        let empty = SnapshotText::new("").unwrap();
        assert_eq!(empty.position(0).unwrap(), Position::new(0, 0));
        assert!(empty.position(1).is_err());
        let ending = SnapshotText::new("x\r\n").unwrap();
        assert_eq!(ending.position(2).unwrap(), Position::new(0, 2));
        assert_eq!(ending.position(3).unwrap(), Position::new(1, 0));
    }
    #[test]
    fn only_exact_requested_missing_method_protocol_envelopes_authorize_fallback() {
        for message in [
            "unknown API method \"getSemanticDiagnostics\"",
            "invalid request: unknown API method \"getSemanticDiagnostics\"",
        ] {
            assert!(missing_method(
                "getSemanticDiagnostics",
                &CorsaError::Protocol(message.into())
            ));
        }
        for message in [
            "user error: unknown API method \"getSemanticDiagnostics\"",
            "unknown API method \"getGlobalDiagnostics\"",
            "getSemanticDiagnostics returned no data",
            "snapshot not found in snapshot registry",
        ] {
            assert!(!missing_method(
                "getSemanticDiagnostics",
                &CorsaError::Protocol(message.into())
            ));
        }
    }
}
