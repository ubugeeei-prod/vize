# P4-7b — Authored content-model route

The SFC batch admits `vue/permitted-contents`: all 40 markup routes now run
through the S2 document. The phase exit remains open for other Relief consumers
and the JSX refusal fallback.

## Source projection

- `vize_s1::parse_with_authored` tokenizes once and constructs the normal surface.
  When that builder implicitly closes a nested HTML `a` or `button`, it also
  constructs the non-repairing authored tree from the same event slice. Both
  trees borrow the original source and satisfy byte fidelity, including holes.
- `S2Template::lower` retains that projection. Ordinary templates reuse their
  existing S1 tree. The semantic S2 lowering and default facade traversal keep
  their existing structure, scopes and hook ordering.
- `MarkupDocument::walk_authored_tree` projects real S1 elements through a typed
  authored element backend. Tags classify as element, component, slot or special
  template by the lint parser's rule. It preserves the written parent relation,
  opening spans, directives, attributes and entity-decoded static values.
  Namespace inheritance controls CDATA rendering. The opening `v-pre` freezes
  names through the producer's existing rewrite; descendants remain frozen and
  a nested `v-pre` remains a plain attribute.
- `authored_document_skeleton` shares the existing skeleton builder and checker.
  `PermittedContents::enter_document` uses that skeleton. The SFC route performs
  no Relief parser re-entry, second tokenization or synthetic template AST.
- Its document hook replaces the original template callback at the same
  registry position, after the shared suppression pre-scan and before element
  traversal. The other 39 routes keep their existing batch ordering. Mixed
  legacy registries, both registration orders and the default preset pin the
  complete diagnostic sequence against a legacy-only content-model wrapper.

The authored walk retains lossless whitespace text. Quirks removes some boundary
whitespace nodes, so complete skeleton row equality is pinned for the structural
battery and semantic boundary witnesses; whitespace/entity witnesses instead
pin complete diagnostics. The normal semantic text traversal remains available
for consumers that require condensed rendered text.

## Evidence

- Patina library: 3,021 passed; one existing ignored test. The 40-rule SFC battery
  compares complete diagnostic structures against the retained template path,
  including message, severity, primary span, label spans, help and fixes, over
  committed rule fixtures and all 90 construct templates.
- 2,592 pair/triple nestings compare the entire authored skeleton with the
  non-repairing template reference. Additional witnesses cover table controls,
  named/dynamic slots, components, built-ins, namespaces, bound attribute facts,
  dynamic content, entity decoding, `v-pre` and foreign CDATA.
- Authored element-query traces compare classification, bindings/modifiers,
  static values, directives and source ranges. A document-only route canary
  panics if `permitted-contents` reaches the legacy template hook.
- Full diagnostic comparisons cover English/Japanese/Chinese, full/hidden help,
  warning overrides, suppression comments and Unicode/CRLF SFC offsets.
- `level_authored_markup_budget` runs alone because allocation counters are
  global. Ordinary source parse adds zero heap calls, peak bytes or arena bytes;
  the warmed default facade keeps exactly 7 content-model allocations and zero
  for every other markup rule. The existing benchmark ceiling is unchanged.
- Storage review: the two added `Vec<SurfaceError>` return annotations in
  `parse.rs` increase arena-vector bound uses from 7 to 9; they return the same
  tokenizer error vector and introduce no additional vector ownership.
- S1 projection tests pin normalized/authored parent relations, byte fidelity,
  ordinary-tree reuse and unchanged tokenizer diagnostics on malformed input.

The PR Check workflow gates the complete Rust suites, JS inventories, formatting,
Clippy and the source length ratchet. The HTML content-model oracle remains the
existing independent Chromium/Lean contract; this change preserves its checker.
