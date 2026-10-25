# Ordered object prop sources (#10458)

`Layout.vue.txt` and `module.expected.txt` pin the complete SSR product module.
The SSR fixture preserves the existing ordered merge contract. Client Vapor
behavior is checked separately; this case claims no native product-route switch.

`jsx.public-0.560.0.before.json` retains the complete public compiler packets
captured from `@vizejs/native@0.560.0` for the six authored JSX sources. The new
source-built complete output snapshot lives in
`crates/vize_atelier_jsx/tests/snapshots/ordered_spread_props__ordered_object_source_modules_and_templates_are_complete.snap`.
Existing corpus rows and historical packets remain unchanged.

The mounted JSX oracle checks complete DOM, text, diagnostics, event receipts,
node identity, and full unmount. The new rows pin both static/object orders,
multiple sources around a bound value, removal/restoration, nullish and false
object sources, class/style composition, and listener side-effect order. The
getter witness pins exactly three reads in authored order and one combined
runtime prop publication per reactive effect. The old mixed-source output
performed two independent object patches and one named prop patch; the fixed
output performs one combined object patch, preserving every source.

`davinci_mounted_behavior/retained_spreads.rs` checks the shared contract across
VDOM, native Vapor and retained Vapor template compilation. Existing native
class/style coverage now requires the retained lane to produce the same complete
mounted trace as well.
