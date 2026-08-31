// The artifact's prose: its generated-file preamble and the column-derivation
// contract. The derivation text quotes the parsed hook list and the dialect
// name prefixes rather than restating them, so the prose tracks the code.

import { OVERRIDES_REL, REGEN_COMMAND } from "./rule-parity-paths.mjs";
import { DIALECT_NAME_PREFIXES } from "./rule-parity-rules.mjs";

export function preambleSection() {
  const lines = [];
  lines.push("<!-- GENERATED FILE — do not edit by hand.");
  lines.push(`     Regenerate: ${REGEN_COMMAND}`);
  lines.push("     Verify:     rust-script tools/commands/davinci/rule-parity.rs --check");
  lines.push("     Generator:  tools/davinci/rule-parity.mjs");
  lines.push(`     Overrides:  ${OVERRIDES_REL} (hand-corrections; applied last) -->`);
  lines.push("");
  lines.push("# Rule-parity matrix (SFC × JSX)");
  lines.push("");
  lines.push(
    "Per-rule registration surface, SFC/JSX path membership, croquis usage, and" +
      " a first-cut portability classification for every lint rule under" +
      " `crates/vize_patina/src/rules/**` — the classification substrate for" +
      " charter #7's fairness metric (one rule corpus serving SFC and JSX at" +
      " parity) and the fact-adoption target in charter #35 (Davinci P0-8).",
  );
  lines.push("");
  return lines;
}

export function derivationSection(model) {
  const lines = [];
  lines.push("## How each column is derived");
  lines.push("");
  lines.push(
    "- **rule identity** — the `name` field of the file's `static META:" +
      " RuleMeta | ScriptRuleMeta | CssRuleMeta | MuseaRuleMeta`. Every rule" +
      " file declares exactly one META; a file with several `impl … for` preset" +
      " variants sharing that META (e.g. `HtmlSelfClosing` +" +
      " `HtmlSelfClosingNuxt`) is one rule.",
  );
  lines.push(
    "- **surfaces** — from the traits the file implements and the hooks each" +
      " impl overrides: `template-ast` (any `Rule` template-visitor hook — the" +
      " hook list is parsed from `trait Rule` in `src/rule.rs`, currently " +
      model.templateHooks.map((h) => "`" + h + "`").join(", ") +
      "), `sfc-source` (`run_on_sfc`), `markup-facade` (`impl MarkupRule`," +
      " the zero-copy IR in `src/markup.rs`), `type-aware-corsa` (listed in" +
      " `TYPE_AWARE_RULES` in `linter/native_type_aware.rs` — served by the" +
      " corsa type-checker session, native-only), `script-oxc` /" +
      " `script-source` (`impl ScriptRule`, split on `uses_ast`/" +
      "`check_program`), `css-text` (`impl CssRule`), `musea-blocks` (musea" +
      " family).",
  );
  lines.push(
    "- **lint() (SFC path)** — a template-family rule runs iff one of its impl" +
      " types is registered into a `RuleRegistry` (every `register…(Box::new(…))`" +
      " site across `vize_patina/src` is collected mechanically); `lint_sfc`" +
      " then drives `run_on_sfc` plus the template visitor over every" +
      " registered rule — and, for `TYPE_AWARE_RULES`, the corsa session" +
      " (`no-op-hooks` cells cannot occur: a hook-less rule reaches SFC only" +
      " via corsa, shown as `corsa`). Script rules run iff listed in" +
      " `linter/script_rules/registry` (dispatched per `<script>` block; the" +
      " same registry also serves plain-JS `lint_script` and inline" +
      " `<script>` in `lint_standalone_html`). CSS rules run iff listed in" +
      " `ALL_BUILTIN_CSS_RULE_NAMES` (dispatched per `<style>` block). Musea" +
      " rules run only under `MuseaLinter` (Art files) — neither path here.",
  );
  lines.push(
    "- **lint_jsx() (JSX path)** — derived from the three-lane partition in" +
      " `Linter::lint_jsx` (`linter/engine.rs`): `ir` = overrides" +
      " `as_markup_rule` → runs over the zero-cost OXC projection;" +
      " `ir-lowered` = additionally `jsx_needs_lowering() == true` → runs over" +
      " the lowered relief AST via the markup visitor; `fallback` = no markup" +
      " projection but has template-visitor hooks → runs over the lowered" +
      " relief AST via the legacy visitor (`legacy_keep_mask`). A registered" +
      " rule with no markup projection and no template-visitor hook —" +
      " `run_on_sfc`-only rules and corsa-only rules — is dispatched but can" +
      " never fire (`lint_jsx` calls neither `run_on_sfc` nor the corsa" +
      " session) — shown as `no (no JSX-reachable hooks)`. Script/CSS" +
      " registries, the corsa session, and `MuseaLinter` are never invoked" +
      " from `lint_jsx`; the generator asserts those absences (and the lane" +
      " anchors) against the current dispatch source and fails on drift.",
  );
  lines.push(
    "- **croquis** — symbol-aware, per P0-7's use-declaration resolution:" +
      " `use vize_croquis::…` aliases (braces, `as`, module aliases) are" +
      " resolved per file and reference sites counted in comment/string-" +
      "stripped code (`direct N`), plus `.analysis()` / `.has_analysis()`" +
      " calls on receivers typed `LintContext` / `MarkupContext` /" +
      " `MarkupDocument` — the lane croquis facts reach rules through" +
      " (`ctx N`). No raw text matching.",
  );
  lines.push(
    "- **classification** — heuristic, precedence container > dialect >" +
      " neutral: **container-bound** if the file touches SFC block structure" +
      " (`SfcDescriptor`/`Sfc*Block`/`parse_sfc`/`sfc_descriptor`/" +
      "`vize_atelier_sfc`), implements `run_on_sfc`, appears in the engine's" +
      " `SHARED_SFC_DESCRIPTOR_RULES`, or is a musea rule (Art-file blocks);" +
      " **vue-dialect-bound** if it references directive-specific node kinds" +
      " (`DirectiveNode`, `ForNode`, `IfNode`, `InterpolationNode`," +
      " `MarkupDirective`, `PropNode::Directive`, …), implements a" +
      " directive-shaped hook (`check_directive`/`check_for`/`check_if`/" +
      '`check_interpolation`/`enter_directive`), contains a `"v-…"` string' +
      " literal, references Vue-only semantics (compiler macros, lifecycle" +
      " names, `is_builtin_component`, `VueDialect`), or belongs to a" +
      " Vue-ecosystem name prefix (" +
      DIALECT_NAME_PREFIXES.map((p) => "`" + p + "/`").join(", ") +
      "); otherwise **neutral-core-candidate**. Overrides from" +
      ` \`${OVERRIDES_REL}\` are applied last; overridden rows are marked \`*\`.`,
  );
  lines.push("");
  lines.push("### Heuristic limitations");
  lines.push("");
  lines.push(
    "- Signals are lexical. A rule that _mentions_ a directive node in shared" +
      " helper code is classified dialect-bound even if its core check is" +
      " neutral; conversely Vue semantics reached through helpers in _other_" +
      " files are invisible. Record corrections in the overrides sidecar, not" +
      " here.",
  );
  lines.push(
    "- `check_interpolation`/`InterpolationNode` count as dialect signals" +
      " (mustache syntax) even though interpolation has a JSX analogue; the" +
      " markup-facade `enter_interpolation`/`enter_conditional`/`enter_list`" +
      " hooks count as neutral (they abstract both syntaxes).",
  );
  lines.push(
    "- Path membership is dispatch-shape, not runtime effect: a rule that" +
      " data-gates on `ctx.analysis()` (see `SEMANTIC_TEMPLATE_RULES`) may" +
      " no-op on JSX when the engine computes no analysis, and a `fallback`" +
      " rule checking Vue-only syntax simply never matches in JSX — both still" +
      " _run_ there. `#[cfg]`-gated registrations (e.g. type-aware rules," +
      " native-only) count as registered.",
  );
  lines.push("- Macro-generated code is invisible to source parsing.");
  lines.push("");
  return lines;
}
