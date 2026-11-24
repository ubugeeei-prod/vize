# P6-7 — Native pre-canonical S2 transform contract

This is the bounded compile-transform contribution to P6-7. It does not
complete the SDK package, the other hook families, or external acceptance.

## Producer and consumer

`compileWithTransformPlugins(template, plugins, options)` parses the original
template through S1 and lowers it to the real arena-resident S2 artifact.
Every plugin sees the same **single pre-canonical boundary**, in registration
order. The next plugin sees the preceding plugin's edited attribute values.
Only after the last hook does the host construct DOM transform facts and emit
through `run_dom_transform_with_profile` and `emit_dom_with_options`.

The native reader numbers nodes in S2 page order, including attached bindings
and branch/loop regions. The applied-edit walk asserts its final count against
`Lowered::op_count`. JSON is a projection and a typed edit request, never an AST
ownership transfer or arbitrary serialized Rust mutation.

## Wire schema 1

Input:

```json
{
  "schema": 1,
  "stage": "s2-precanonical-static-attributes",
  "plugin": "design-system",
  "file": "src/Button.vue",
  "nodes": [
    {
      "id": 0,
      "kind": "ui.element",
      "tag": "button",
      "namespace": "html",
      "attrs": [{ "name": "class", "value": "legacy-btn" }]
    }
  ]
}
```

Output:

```json
{
  "schema": 1,
  "edits": [
    {
      "kind": "replace-static-attribute",
      "node": 0,
      "name": "class",
      "value": "ds-button"
    }
  ]
}
```

The input and output attribute values are semantic text. The host decodes the
original S2 spelling for the reader, encodes new values into the S2 arena, and
the ordinary canonical consumer decodes them once. `null` means a bare static
attribute; `value` is required. Original owner/attribute spans and node IDs are
preserved. Each edit adds a `plugin.transform:<name>` provenance record.

## Validation and failure

Only an **existing**, unique static attribute on a native HTML element may
change: `class`, `id`, `title`, `role`, `alt`, `data-*`, and `aria-*`. Component,
SVG/MathML, template-carrier, structural attribute, unknown node, missing
attribute, duplicate edit, unknown field, and unknown schema requests refuse.
Limits are 1 MiB source, 32 plugins, 1 MiB reply, 256 edits per plugin, and
4096 bytes per value; NUL values refuse. Identities and cache inputs also have
explicit string/count limits. Validation precedes all arena edits.

Callbacks are synchronous. On each cache miss the same batch is called twice;
unequal typed replies refuse. Callback failures, invalid source, unsupported
canonical emission, and parity failures return errors without a compile result.
No partially transformed source is published or compiled through a fallback.

## Code, facts and maps

This entry emits module code with identifier prefixing. Options are `filename`,
`sourceMap`, `hoistStatic`, `cache`, and `cacheDir`; hoisting defaults to false.
It returns `{ result: CompileResult, plugins: Cost[] }`.

S2 canonical facts are fresh products of the edited artifact, never cached
across hook invocations. Lowering's structural/scope/text tables remain valid:
the edit language changes no structure, bindings, expressions, or text runs.
Hoisted declarations must contain the transformed values too.

The map bridge parses the **original** compatibility tree and mirrors only the
validated typed attribute edits before its canonical transform. It never
constructs and reparses an edited source string. Native S2 code/preamble must
match that independent compatibility consumer exactly; otherwise compilation
refuses. The map therefore describes the selected native code while retaining
the original authored template and original attribute anchors. Map requests do
not select an untransformed fallback.

## Cache and attribution

The content key covers the complete original source, filename, preceding edited
S2 projection, schema/stage, host build/features, compile options, plugin name/version/fingerprint, and
sorted declared `cacheInputs`. Opt-in caching requires an explicit list, even
when empty. Every hit validates its edits against the current native artifact
and reruns canonical compilation and mapping. Only audited edits after a
successful complete compile enter the cache.

The memory store is FIFO, at most 64 entries and 1 MiB serialized edit payload.
`cacheDir` adds atomic persistent records in `s2-transforms-v1`; torn, wrong-key,
unsupported, oversized and invalid records miss. Each disk record is at most
1 MiB. Disk retention follows the caller's cache-directory cleanup policy.
Per-plugin costs report nodes, edits, cache status, content key, total hook time,
and the time inside both JS calls; cached calls report zero JS time.

## Witnesses

- `cargo test -p vize_vitrine --lib plugin_transform`: actual design-system
  rewrite, full code/maps with hoisting on/off, original Unicode source,
  semantic entity values, page IDs through branches/loops/bindings, atomic
  refusal, deterministic audit, identity/input cache misses, invalid disk data.
- `tests/tooling/davinci-plugin-transform.test.ts`: NAPI callback, selected code,
  decoded map segments, same-process reuse/code miss and strict refusal.

General AST rewrites, expression replacement, insertion/deletion, other targets,
production fact demands in transform batches, and asynchronous callbacks are
outside this explicitly versioned bounded contract.
