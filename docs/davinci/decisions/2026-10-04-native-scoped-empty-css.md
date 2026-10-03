# Literal scoped empty selectors from original CSS tokens

This bounded native SFC slice accepts one literal `.class:empty` rule in an
original scoped CSS block. The existing parse-once StyleSyntax owner retains
the actual cssparser qualified-rule prelude and declaration callbacks. A private
borrowed EmptyClassStyle receipt requires its fixed original dot, classname,
colon and literal `empty` identifier token window. It lends those actual tokens
and their whole-file spans; caller-supplied offsets or pseudo metadata cannot
construct it. There is no additional parser, stylesheet walk or selector AST.

The native consumer inserts the validated component scope attribute at the
actual classname token end, before the original colon. All other emitted CSS
bytes, including leading comments, Unicode, CRLF and final pre-brace whitespace,
retain their original source spans. The generated scope attribute remains
unlinked. Optional CSS, trim, original block order, default filename identity,
module attachment and complete original script/template/File custody remain
the accepted result contract. The actually merged single-class and class-list
families, including #7608/#7609, remain unchanged.

Literal classnames and literal lowercase `empty` are required. Escaped names,
uppercase or repeated pseudos, pseudo elements/functions, lists/combinators,
internal whitespace/comments and trailing selector comments refuse. Existing
declaration syntax/binding/quoted-token refusals stay authoritative. Broader
selectors or values, multiple/nested rules, Modules, preprocessors, external
or custom blocks, SSR scoped output and whole-product/history gates remain
unfinished. A refused block retains original syntax and complete descriptor
custody and produces no partial CSS/module success.

Before implementation, pinned Vue 3.5.35 was audited with complete original
SFC source maps: `.a:empty` becomes `.a[data-v-empty]:empty`, and every other
byte is preserved in the admitted family. Comments adjacent to the pseudo have
different insertion behavior and remain refused. Actual Chromium, using the
stock Vue module from an original `let text=''` SFC, verified empty/nonempty/
empty transitions, computed background and exclusion outside the component.
This primary audit is separate from current-source native Rust acceptance.

Three authored original-SFC/full-module/CSS fixtures cover a literal class,
Unicode/CRLF/comment trivia and original setup with the actual default scope
identity. Complete stock Vue CSS and source-map goldens retain the entire
original SFC in sourcesContent. Separate dev-only ordinaryCss goldens retain
the ordinary printer's selector whitespace contract. Rust laws compare complete
native results, recording-disabled output and custody across moves, independently
decode whole-source map coordinates, and prove that every original emitted CSS
byte is linked with exactly one generated scope gap. Trim and refusal laws
retain the same obligations.

The existing guaranteed protected tooling-shard-1 scoped action captures fresh
Rust modules/CSS/maps for all previously accepted single-class/list fixtures
and these three empty fixtures. The new Node harness requires the promised
capture file, compares complete results, then mounts those exact modules through
Vue 3.5.35 in Chromium. Every fixture proves empty/nonempty/empty selector and
computed CSS behavior, original setup mutation where present, scope attributes,
outside exclusion and unmount. Current-source and browser receipts are retained
as action artifacts. Authored-fixture local execution is explicitly separate
and cannot satisfy the fresh Rust acceptance condition.
Runtime resolution rewrites only dev-oracle parser-certified top-level import
source ranges; original setup string/comment controls containing `from "vue"`
remain byte-exact. The browser cache restores executables, while Chromium OS
dependencies are installed on every fresh protected runner, including cache hits.
Custom attributes retain the original descriptor UnsupportedAttribute refusal
and complete container bytes without constructing an admitted CSS receipt.

One independent conventional PR carries this complete bounded feature because
its genuine provider dependencies have already merged. Exact-head Actions,
the guaranteed protected fresh native/browser capture, complete suites and all
100 unchanged instruction ceilings must pass before actual merge is delivery.
Ordinary routes, default selection, fix-history gates and budgets stay unchanged.

The initial #7650 queue candidate e9d1161f52506eb5cbcf4ccdea5416080f0efa60
ran Check 37156718174 from source fb15ffb7c129d6a395445817472e29fddb0054d0.
Its guaranteed scoped step succeeded: all four single-class, three class-list
and three empty current-source Rust captures match complete committed results,
and actual Chromium 151.0.7922.34/Vue 3.5.35 passed the empty/runtime controls.
Artifact 11285308621 retains these source-qualified positive receipts.
The global JS job 111301605725 failed before build/history: the pinned Wild
0.9.0 x86_64 release asset returned HTTP 503 across all five curl retries and
exited 22. This is not a source or native-history failure. The known-red entry
was actually dequeued, with source unchanged and queue/auto-merge both null.
These scoped positives are not whole-candidate acceptance; targeted existing
failed-gate recovery and a fresh protected candidate remain required.
