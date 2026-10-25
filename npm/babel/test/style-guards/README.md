# Additive style guards

This CSR-only witness is separate from the unchanged eight supported SSR,
mount, and hydration rows. It runs the packed plugin through Babel 7 and 8,
each through its ESM and CommonJS entrypoint, with an explicit active source
native preparation and the real rc.11 browser runtime.

The eight CSSOM vectors retain the exact source inputs from the shared style
compiler fixture. Four additional `STYLE`/`Style` attribute-order cases preserve
case-sensitive authored bytes and test case-insensitive HTML attribute behavior.

Refused inputs compare complete raw DOM, computed styles, priorities, setup,
trusted events, updates, and disposal with a genuine dynamic-style template.
Admitted inputs compare raw DOM with a genuine static-style template and compare
computed styles/priorities independently with the dynamic CSSOM path. Both raw
style strings remain in the artifact; no whitespace or punctuation is normalized.

Every JSX file calls the real native compiler once. Its whole DTO, source image
identity, printed input, original source map, final host map, and actual style
operation frontier are retained. Folded JSX template names and aggregate style
expressions have independent decoded native-map anchors; the final Babel map
also retains the complete original authored source.

Run inside the active source preparation with the usual packed-plugin/native
inputs and a fresh `VIZE_BABEL_STYLE_GUARD_OUTPUT`:

```sh
node npm/native/scripts/test-preparation.mjs node --test npm/babel/test/style-guards.test.mjs
```
