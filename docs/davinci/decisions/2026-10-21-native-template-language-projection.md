# Native template language projection (#6837)

The public SFC parser now projects the canonical optional `lang="html"`
attribute from the existing L1 Vue template-only owner. The projection borrows
the original attribute value and content bytes, and preserves the retained
public descriptor's `lang`, `attrs`, filename, source, and absolute locations.

This bounded family still requires one ordinary template, default parse policy,
and no script, style, custom block, external source, duplicate attribute,
encoded language, or uncertain boundary. Those inputs keep the existing
Croquis route and its complete diagnostics. The allocation-free L1 observation
remains the only authority for admission; the public adapter does not rescan or
decode the attribute.

The pinned official Vue fixture identifies the newly admitted original input.
Complete retained-parser comparisons cover the descriptor and all DOM inline,
DOM module, Vapor, SSR, and source-map profiles. The controlled whole-SFC work
test includes the language-header input in cold and warm observations.

This advances the default product route for #6837. It does not establish the
full SFC descriptor migration, native-only compilation, or completion of the
parent Davinci roadmap.
