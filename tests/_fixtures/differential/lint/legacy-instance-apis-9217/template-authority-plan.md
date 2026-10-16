# Retained template reference authority required before acceptance

Child #9251 preserves the old implementation for review; its synthetic reparsing
and boolean shadow walker are not accepted. Parent #9222 is script-only. All
70 original contracts and all evidence remain intact. The 17 existing template
controls do not certify the missing boundaries below.

| Concern | Existing producer or owner | Required contract |
| --- | --- | --- |
| Actual expression/slot AST | `vize_relief::SimpleExpressionNode::js_ast`, `RetainedJsAst::{as_expression,as_slot_parameters}` | Consume the admitted role. Never parse a missing/failed payload inside the rule. |
| Full identifier token | `vize_l2::resolution::Occurrence` | Retain cooked name and full AST token span; an offset plus name length cannot handle escaped identifiers. |
| Wrapper/entity coordinates | `vize_l1::embed::syntax::{RetainedExpression,RetainedSlotParams,RetainedHandlerBody}::authored_span`; `vize_l2::expr::JsExpr::ast_span_to_source` | Convert through the actual checked owner, including entity-decoded attributes; no source-name searches or equal-length assumptions. |
| Lexical ownership | Croquis template `ids.rs::check_expression_refs` and `occurrences.rs::note_expression_occurrences` | Add demand-only reference evidence during the existing semantic walk: role, full token/authored span, bound/component-context/unknown owner and completeness/refusal. |
| Slot parameters | `visit_element/first_pass.rs`; core `steps/legacy.rs` | Use retained parameter roles and dialect-gated legacy slot-scope conversion, including destructuring/default initializers. |
| v-for collection | `NativeForInput::{aliases,collection,alias_authored_span,collection_authored_span}` | Resolve collection in the outer scope before introducing aliases for the body. |
| Transformed conditions | `directives/control_flow.rs::visit_if` | Visit each branch condition and user key, then children; also handle raw directives. |
| Refusal/guard | `vize_l2::resolution::walk::budget` | Preserve depth64/node4096 refusal and rollback; bound or iteratively traverse the template tree too. Unknown facts cannot become diagnostics. |

A demand-only original-walk packet is needed; merely replacing the local parser
with a local AST walker would still leave ownership gaps. Preserve parse-once
ownership and existing serialized Croquis contracts. The consumer reports only
positively witnessed component-context value reads, suppresses proven local
bindings and leaves unknown coverage explicit.

Reuse limits:

- SliceA `has_authored_template_binding_read` proves a read of one exact setup
  declaration. Absence does not prove an unbound component global.
- `FileArtifact::template_symbol_at_offset` currently admits attached Handler /
  OriginalFor sites, not every template expression.
- `resolve_expression` rejects MissingBinding. Never invent a BindingId for
  `$listeners` to bypass that authority.
- Croquis transformed `visit_for` currently introduces aliases before scanning
  the source; that ordering cannot prove `$listeners in $listeners` semantics.

Add controls for entity-encoded and escaped identifiers, comments/strings/types,
callback/slot shadowing, legacy slot-scope, collection/body scope order,
transformed If conditions, statement handlers, malformed payloads, traversal
refusal and unchanged parse counters. Reuse existing Croquis slot/legacy tests,
L1 parameter authority tests, L2 coordinates/for-head/query tests and the
component-template-runtime-reads corpus. Keep this child Draft until those
contracts and fresh native/source qualification pass.
