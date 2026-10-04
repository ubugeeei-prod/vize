# Honor existing property-quote policies (#7753)

Issue: [#7753](https://github.com/ubugeeei-prod/vize/issues/7753).
Configuration roadmap: [#6098](https://github.com/ubugeeei-prod/vize/issues/6098).

Source audit on actual main `6987c523` finds an existing configuration mismatch:
`quoteProps` is present in schema/generated config types, Rust, CLI/editor and
WASM serde projection, but the final Rust-to-Oxc conversion omits the field.
Pinned Oxc `JsFormatOptions::default()` and Glyph both use AsNeeded. Map the
existing AsNeeded/Consistent/Preserve variants in that existing constructor;
public defaults and option scope remain unchanged. The constructor stays under
the 350-line owner limit. No new stage or serialization is introduced.

For `const value={"name":1,"needs-dash":2}\n`, the independently derived Preserve
reference is `const value = { "name": 1, "needs-dash": 2 };\n` (46 bytes).
Current static AsNeeded projection implies 44 bytes with an unquoted name;
that statement is a source inference, not an executed baseline observation.
Actual source-built Actions must prove the complete references before runtime
acceptance. Test Consistent with and without a sibling requiring quotes, Preserve
with quoted and unquoted siblings, and explicit/default AsNeeded controls.

Public Rust script, explicit TS/TSX/JSX and Vue 2/2.7/3 SFC laws compare complete
independent output and three-pass fixed points. One additive configured CLI case
pins its input/config/full reference, and the actual source-built CLI must match
all cases. Preserve original corpus/history references, 300 API plans and all104
unchanged instruction ceilings. No native provider acceptance credit is granted.
Source inputs/options stay immutable; result/source-map APIs are unchanged.
After Stack #7748 actually merged at `19be0e24`, replay retains all nine prior
cases byte-for-byte and adds the property-quote case as the tenth. The complete
NAPI owner transition and original nine plans/25 calls remain unchanged.

This slice is independent of raw separator/Node Stack #7748, prepared in its own
wt worktree from actual main. The v0.431.0 publication freeze has been lifted;
fresh source Actions and all104 must pass, then use protected queue through actual
merge and verify final reporter credit under the existing delivery-control policy.
The source and PR carry the verified reporter trailer; GitHub may normalize a
same-account reporter to the verified primary author in a single-commit squash.
Replay fixture
metadata on actual main when other ready slices land; do not drop their cases.

TODO: broader profiles, option combinations/source maps remain open under #6098.
Separate `jsxSingleQuote` projection and Node quote-option exposure remain outside
this repair; the Node public type does not yet promise either quote option.
