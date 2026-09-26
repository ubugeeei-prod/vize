//! Restricted edits: whitespace preserves the parsed program; output adds comments.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    reason = "serialized plugin boundary uses standard strings"
)]

use crate::CompileResult;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use serde::Deserialize;
use vize_l0::Allocator;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Edit {
    pub start: u32,
    pub end: u32,
    pub text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Addition {
    placement: String,
    comment: String,
}

pub(super) fn apply(
    compiled: &CompileResult,
    family: &str,
    response: &str,
) -> Result<(CompileResult, u32), String> {
    if response.len() > 1024 * 1024 {
        return Err("output response exceeds 1 MiB".to_owned());
    }
    let mut operations = None;
    let edits: Vec<Edit> = match family {
        "formatter" => {
            serde_json::from_str(response).map_err(|e| format!("invalid formatter edits: {e}"))?
        }
        "output" => {
            let additions: Vec<Addition> = serde_json::from_str(response)
                .map_err(|e| format!("invalid output additions: {e}"))?;
            if additions.len() > 1024 {
                return Err("output hook exceeds 1024 additions".to_owned());
            }
            operations = Some(additions.len() as u32);
            let mut prefix = String::new();
            let mut suffix = String::new();
            for addition in additions {
                if addition.comment.contains("*/") || addition.comment.contains('\0') {
                    return Err(
                        "output comments may not contain a closing delimiter or NUL".to_owned()
                    );
                }
                if [
                    "sourceMappingURL",
                    "sourceURL",
                    "__PURE__",
                    "__NO_SIDE_EFFECTS__",
                ]
                .iter()
                .any(|directive| addition.comment.contains(directive))
                {
                    return Err(
                        "output comments may not introduce map or optimization directives"
                            .to_owned(),
                    );
                }
                let text = format!("/* {} */\n", addition.comment);
                match addition.placement.as_str() {
                    "prepend" => prefix.push_str(&text),
                    "append" => {
                        suffix.push('\n');
                        suffix.push_str(&text);
                    }
                    _ => return Err("output placement must be prepend or append".to_owned()),
                }
            }
            let mut edits = Vec::new();
            if !prefix.is_empty() {
                edits.push(Edit {
                    start: 0,
                    end: 0,
                    text: prefix,
                });
            }
            if !suffix.is_empty() {
                edits.push(Edit {
                    start: compiled.code.len() as u32,
                    end: compiled.code.len() as u32,
                    text: suffix,
                });
            }
            edits
        }
        _ => return Err("unknown output hook family".to_owned()),
    };
    if edits.len() > 1024 {
        return Err("output hook exceeds 1024 edits".to_owned());
    }
    let mut prior_end = 0;
    let mut prior_start = None;
    let mut code = String::new();
    for edit in &edits {
        if edit.start > edit.end || edit.start < prior_end || prior_start == Some(edit.start) {
            return Err(
                "output edits must be ordered, nonoverlapping and have unique starts".to_owned(),
            );
        }
        let old = compiled
            .code
            .get(edit.start as usize..edit.end as usize)
            .ok_or_else(|| {
                "output edit is outside the code or splits a UTF-8 character".to_owned()
            })?;
        if family == "formatter" && (!whitespace(old) || !whitespace(&edit.text)) {
            return Err("formatter edits may only replace ASCII whitespace".to_owned());
        }
        code.push_str(
            compiled
                .code
                .get(prior_end as usize..edit.start as usize)
                .ok_or_else(|| "invalid output edit boundary".to_owned())?,
        );
        code.push_str(&edit.text);
        prior_end = edit.end;
        prior_start = Some(edit.start);
    }
    code.push_str(compiled.code.get(prior_end as usize..).unwrap_or_default());
    if code.len() > compiled.code.len().saturating_add(1024 * 1024) {
        return Err("output hook adds more than 1 MiB".to_owned());
    }
    if family == "formatter" && normalized(&compiled.code)? != normalized(&code)? {
        return Err("formatter changed JavaScript semantics, literals or comments".to_owned());
    }
    if family == "output" && normalized(&compiled.code)?.0 != normalized(&code)?.0 {
        return Err("output additions changed JavaScript semantics".to_owned());
    }
    if edits.is_empty() {
        super::map::rebase(compiled, &compiled.code, &[])?;
        return Ok((compiled.clone(), 0));
    }
    let mut result = compiled.clone();
    result.map = super::map::rebase(compiled, &code, &edits)?;
    result.code = code;
    Ok((result, operations.unwrap_or(edits.len() as u32)))
}

fn whitespace(text: &str) -> bool {
    text.bytes()
        .all(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
}

fn normalized(code: &str) -> Result<(String, Vec<String>), String> {
    let allocator = Allocator::new();
    let parsed = Parser::new(&allocator, code, SourceType::unambiguous())
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..Default::default()
        })
        .parse();
    if !parsed.diagnostics.is_empty() || parsed.panicked {
        return Err("formatter input or output is not valid JavaScript".to_owned());
    }
    let comments = parsed
        .program
        .comments
        .iter()
        .map(|comment| {
            code.get(comment.span.start as usize..comment.span.end as usize)
                .unwrap_or_default()
                .to_owned()
        })
        .collect();
    Ok((
        Codegen::new()
            .with_options(CodegenOptions::minify())
            .build(&parsed.program)
            .code,
        comments,
    ))
}
