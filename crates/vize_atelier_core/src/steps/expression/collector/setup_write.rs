//! Original-AST mutable setup writes within the existing identifier walk.

use oxc_ast::ast::{
    AssignmentExpression, AssignmentTarget, IdentifierReference, SimpleAssignmentTarget,
    UpdateExpression,
};
use oxc_span::GetSpan;
use vize_l0::{String, cstr};

use super::IdentifierCollector;
use crate::options::BindingType;

impl IdentifierCollector<'_, '_> {
    fn guarded_setup_write_kind(&self, name: &str) -> Option<BindingType> {
        if self.is_local(name)
            || self.ctx.is_in_scope(name)
            || !self.ctx.options.inline
            || self.ctx.options.ssr
            || self.ctx.options.vapor
        {
            return None;
        }
        self.ctx
            .options
            .binding_metadata
            .as_ref()
            .filter(|table| table.is_script_setup)
            .and_then(|table| table.bindings.get(name).copied())
            .filter(|kind| matches!(kind, BindingType::SetupLet | BindingType::SetupMaybeRef))
    }

    pub(super) fn guard_setup_assignment(&mut self, expr: &AssignmentExpression<'_>) {
        let AssignmentTarget::AssignmentTargetIdentifier(ident) = &expr.left else {
            return;
        };
        let Some(kind) = self.guarded_setup_write_kind(ident.name.as_str()) else {
            return;
        };
        let start = expr.span.start as usize;
        let right = expr.right.span();
        let right_start = right.start as usize;
        let end = expr.span.end as usize;
        let Some(head) = self.source.get(start..right_start) else {
            return;
        };
        let Some(source) = self.source.get(right_start..end) else {
            return;
        };
        // Child visits already produced every RHS edit. Reuse those exact
        // original coordinates; no second AST walk or parse is introduced.
        let prefixes = self
            .rewrites
            .iter()
            .filter(|(at, _)| *at >= right_start && *at <= end)
            .map(|(at, text)| (*at - right_start, text.clone()))
            .collect();
        let suffixes = self
            .suffix_rewrites
            .iter()
            .filter(|(at, _)| *at >= right_start && *at <= end)
            .map(|(at, text)| (*at - right_start, text.clone()))
            .collect();
        let right = super::super::splice::splice_insertions(source, prefixes, suffixes, 0);
        let plain = cstr!("{head}{right}");
        self.finish_setup_write(expr.span.start, expr.span.end, ident, kind, plain);
    }

    pub(super) fn guard_setup_update(&mut self, expr: &UpdateExpression<'_>) {
        let SimpleAssignmentTarget::AssignmentTargetIdentifier(ident) = &expr.argument else {
            return;
        };
        let Some(kind) = self.guarded_setup_write_kind(ident.name.as_str()) else {
            return;
        };
        let Some(plain) = self
            .source
            .get(expr.span.start as usize..expr.span.end as usize)
        else {
            return;
        };
        let plain = String::from(plain);
        self.finish_setup_write(expr.span.start, expr.span.end, ident, kind, plain);
    }

    fn finish_setup_write(
        &mut self,
        start: u32,
        end: u32,
        ident: &IdentifierReference<'_>,
        kind: BindingType,
        plain: String,
    ) {
        let target_end = ident.span.end as usize;
        // Replace only this original target's legacy suffix. Putting .value
        // inside any authored target parentheses preserves both update forms.
        let scanned = target_end
            + self
                .source
                .as_bytes()
                .get(target_end..)
                .unwrap_or_default()
                .iter()
                .take_while(|byte| **byte == b')')
                .count();
        self.suffix_rewrites
            .retain(|(at, text)| *at != scanned || text.as_str() != ".value");
        self.suffix_rewrites
            .push((target_end, String::from(".value")));
        self.rewrites
            .insert((start as usize, cstr!("(_isRef({}) ? ", ident.name.as_str())));
        let end = end as usize;
        // The splicer reverses construction order at one position. An outer
        // tail must therefore precede existing child tails in this list.
        let position = self
            .suffix_rewrites
            .iter()
            .position(|(at, _)| *at == end)
            .unwrap_or(self.suffix_rewrites.len());
        let fallback = match kind {
            BindingType::SetupLet => plain,
            BindingType::SetupMaybeRef => String::from("null"),
            _ => return,
        };
        self.suffix_rewrites
            .insert(position, (end, cstr!(" : {fallback})")));
        self.used_is_ref = true;
    }
}

#[cfg(test)]
mod tests {
    //! Full write expressions, role guards, and actual pinned Vue ref semantics.

    use std::io::Write;
    use std::process::{Command, Stdio};

    use serde_json::{Value, json};
    use vize_l0::{Allocator, FxHashMap};

    use crate::lane::TransformContext;
    use crate::options::{BindingMetadata, BindingType, TransformOptions};
    use crate::steps::expression::rewrite::rewrite_expression;

    fn options() -> TransformOptions {
        let mut bindings = FxHashMap::default();
        bindings.insert("evt".into(), BindingType::SetupLet);
        bindings.insert("attr".into(), BindingType::SetupLet);
        bindings.insert("maybe".into(), BindingType::SetupMaybeRef);
        bindings.insert("n".into(), BindingType::SetupRef);
        TransformOptions {
            prefix_identifiers: true,
            inline: true,
            binding_metadata: Some(BindingMetadata {
                bindings,
                props_aliases: FxHashMap::default(),
                is_script_setup: true,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn original_setup_write_expressions_preserve_exclusive_rhs_and_update_results() {
        let cases = [
            (
                "evt = attr + n",
                "(_isRef(evt) ? evt.value = _unref(attr) + n.value : evt = _unref(attr) + n.value)",
                5,
                5,
                3,
                0,
            ),
            (
                "evt += next(n)",
                "(_isRef(evt) ? evt.value += _ctx.next(n.value) : evt += _ctx.next(n.value))",
                3,
                3,
                3,
                1,
            ),
            (
                "evt ||= next(n)",
                "(_isRef(evt) ? evt.value ||= _ctx.next(n.value) : evt ||= _ctx.next(n.value))",
                1,
                1,
                3,
                0,
            ),
            (
                "evt &&= next(n)",
                "(_isRef(evt) ? evt.value &&= _ctx.next(n.value) : evt &&= _ctx.next(n.value))",
                2,
                2,
                3,
                1,
            ),
            (
                "evt ??= next(n)",
                "(_isRef(evt) ? evt.value ??= _ctx.next(n.value) : evt ??= _ctx.next(n.value))",
                1,
                1,
                3,
                0,
            ),
            (
                "evt = attr = next(n)",
                "(_isRef(evt) ? evt.value = (_isRef(attr) ? attr.value = _ctx.next(n.value) : attr = _ctx.next(n.value)) : evt = (_isRef(attr) ? attr.value = _ctx.next(n.value) : attr = _ctx.next(n.value)))",
                2,
                2,
                2,
                1,
            ),
            ("evt++", "(_isRef(evt) ? evt.value++ : evt++)", 1, 2, 3, 0),
            ("++evt", "(_isRef(evt) ? ++evt.value : ++evt)", 2, 2, 3, 0),
            ("evt--", "(_isRef(evt) ? evt.value-- : evt--)", 1, 0, 3, 0),
            ("--evt", "(_isRef(evt) ? --evt.value : --evt)", 0, 0, 3, 0),
            (
                "((evt)) += next(n)",
                "(_isRef(evt) ? ((evt.value)) += _ctx.next(n.value) : ((evt)) += _ctx.next(n.value))",
                3,
                3,
                3,
                1,
            ),
            (
                "(evt => { evt++; return evt })(7)",
                "(evt => { evt++; return evt })(7)",
                8,
                1,
                3,
                0,
            ),
            (
                "(() => { let evt = 7; evt++; return evt })()",
                "(() => { let evt = 7; evt++; return evt })()",
                8,
                1,
                3,
                0,
            ),
        ];
        let mut rows = Vec::new();
        for (index, (source, expected, value, event, attr, calls)) in cases.into_iter().enumerate()
        {
            let allocator = Allocator::new();
            let ctx = TransformContext::new(&allocator, source, options());
            let actual = rewrite_expression(source, &ctx, false, None);
            assert_eq!(actual.parse_error, None);
            assert_eq!(actual.code.as_str(), expected);
            assert_eq!(actual.used_is_ref, index < 11);
            rows.push(json!({"source": source, "code": actual.code.as_str(),
            "expected": {"value": value, "event": event, "attr": attr, "calls": calls, "n": 2}}));
        }
        let input = serde_json::to_vec(&rows).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut child = Command::new("node")
            .arg(root.join("tests/tooling/support/setup-write-semantics-7881.mjs"))
            .current_dir(&root)
            .env("NODE_ENV", "production")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let write = child.stdin.take().unwrap().write_all(&input);
        let output = child.wait_with_output().unwrap();
        let packet = std::env::var("NEXTEST_PROFILE")
            .ok()
            .map(|profile| root.join("target/nextest").join(profile));
        if let Some(packet) = packet {
            std::fs::create_dir_all(&packet).unwrap();
            std::fs::write(packet.join("setup-write-semantics-input.json"), &input).unwrap();
            std::fs::write(
                packet.join("setup-write-semantics-stdout.json"),
                &output.stdout,
            )
            .unwrap();
            std::fs::write(
                packet.join("setup-write-semantics-stderr.txt"),
                &output.stderr,
            )
            .unwrap();
            std::fs::write(
                packet.join("setup-write-semantics-process.json"),
                serde_json::to_vec(&json!({
            "status": output.status.to_string(), "code": output.status.code(),
            "stdinWriteError": write.as_ref().err().map(ToString::to_string)}))
                .unwrap(),
            )
            .unwrap();
        }
        assert!(write.is_ok(), "original stdin failure: {write:?}");
        assert!(
            output.status.success(),
            "original exit: {}; stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        let evidence: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(evidence["vue"], "3.6.0-rc.9");
        assert_eq!(evidence["observations"].as_array().unwrap().len(), 26);
    }

    #[test]
    fn maybe_ref_setup_writes_use_null_fallback() {
        for (source, expected) in [
            (
                "maybe = next(n)",
                "(_isRef(maybe) ? maybe.value = _ctx.next(n.value) : null)",
            ),
            ("maybe++", "(_isRef(maybe) ? maybe.value++ : null)"),
        ] {
            let allocator = Allocator::new();
            let ctx = TransformContext::new(&allocator, source, options());
            let actual = rewrite_expression(source, &ctx, false, None);
            assert_eq!(actual.parse_error, None);
            assert_eq!(actual.code.as_str(), expected);
            assert!(actual.used_is_ref);
        }
    }

    #[test]
    fn only_original_vdom_inline_setup_write_roles_enable_the_conditional() {
        for (inline, setup, ssr, vapor, expected) in [
            (true, false, false, false, "evt.value = 'mouseup'"),
            (true, true, true, false, "evt.value = 'mouseup'"),
            (true, true, false, true, "evt.value = 'mouseup'"),
            (false, true, false, false, "$setup.evt = 'mouseup'"),
        ] {
            let source = "evt = 'mouseup'";
            let allocator = Allocator::new();
            let mut options = options();
            options.inline = inline;
            options.ssr = ssr;
            options.vapor = vapor;
            options.binding_metadata.as_mut().unwrap().is_script_setup = setup;
            let ctx = TransformContext::new(&allocator, source, options);
            let actual = rewrite_expression(source, &ctx, false, None);
            assert_eq!(
                (actual.code.as_str(), actual.used_is_ref, actual.parse_error),
                (expected, false, None)
            );
        }
        let source = "evt++";
        let allocator = Allocator::new();
        let mut ctx = TransformContext::new(&allocator, source, options());
        ctx.add_identifier("evt");
        let actual = rewrite_expression(source, &ctx, false, None);
        // This inverse role law preserves the existing transform-scope carrier.
        // The new conditional helper has no admission for an external local.
        assert_eq!(
            (actual.code.as_str(), actual.used_is_ref),
            ("evt.value++", false)
        );
    }
}
