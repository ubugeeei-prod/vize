# Supply descriptor inputs and SSR context to the official inspector

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The CLI inspector's official Vue `compileTemplate` call requested SSR without
passing the SFC descriptor's CSS variables. Vue warned about that omission and
compiled Elk's `StatusMedia.vue` without its server-rendered grid variable.
Vize already emitted that variable, so the comparison included a difference
caused by the incomplete reference invocation.

Pass `ssrCssVars: descriptor.cssVars` to the existing official compiler call.
Also pass `slotted: descriptor.slotted` in both the inline DOM script-template
options and the separate template compiler invocation. Without it, Vue assumes
slotted styles exist even when an authored scoped style has no `:slotted()`.
Finally, retain `templateOptions.ssr` when compiling a separate SSR script;
otherwise Vue emits the client CSS-variable helper in the SSR script.

The published `@vitejs/plugin-vue@6.0.7` passes these descriptor fields and SSR
context through its resolved template options to both compiler entry points.
Its implementation digest and relevant locations are retained in the fixture
provenance. The native compiler and all comparison budgets are unchanged. The
existing 749-line inspector does not grow.

The complete upstream StatusMedia source, pre-fix official SSR output,
independent complete DOM/SSR references, provider identity and license are
retained under `tests/_fixtures/differential/compiler/inspector-ssr-css-vars/`.
The focused test executes the inspector's actual embedded script in Node.
It failed with Vue's missing-CSS-variable warning before the change, then
passed the source/provider identity, complete DOM/SSR output, and literal-CSS
control. A later full SSR reference exposes the omitted script context by
requiring the client helper to be absent. The CSS sentinel `:--v4a184587` is intentional;
the official server renderer converts it to `--v4a184587:2;` for value 2.

The same directory retains complete pinned `NavSideItem.vue`, independent full
DOM/SSR outputs, and controls adding one `:slotted()` rule without removing
original bytes. The original reference comparison fails on the missing slot
arguments before the correction. The positive and negative descriptor cases
then pass for both compilation paths; hard-coding either flag cannot pass.

The remaining explicit plugin options were reviewed against all 550 pinned
Elk, VueFes and npmX originals. None uses template language/source attributes,
script sources, authored Vapor or slotted selectors. Supplying the optional
descriptor template AST and source map changes no official SSR code, formatted
code, warning or error packet for Elk or VueFes. Vite asset URL configuration
depends on the host bundler context and is not imported into this raw compiler
comparison. No further omission exercised by this corpus was identified.

## Scope and remaining parity failure

A public v0.426.3 CLI replay regenerated the old 9201/25188 passing SSR counts
on the same 259-file Elk archive and official provider. The compared clean
source snapshot is `cd861aa001f7ea458f8b0147e5b9c850b9db1c87`, with CLI SHA-256
`8723ab2f2d6e99d68cf03bfc88dcd60092e1fa9d873070f73b3edd5a9efe6f93`.
The separately built repair CLI has SHA-256
`91ca413e95ca4c34fd4e74bbde09b2cb1787399df87f6558862af865e2aabd1c`;
its build log is retained, but its exact source SHA is unknown because no
contemporaneous source/tree/dirty receipt was captured. Both executables
produce 9217/25007 before this change, with identical native outputs. This is
not a verification of moving main. The intervening differences include shipped
props-rest, page-metadata, nested-await and CSS-variable fixes; this change
does not revert them.

The CSS-variable input alone gives 9217 additions and 25010 removals. Only
StatusMedia changes, from 20/25 to 20/28. The full descriptor/context correction
also changes scoped slot output and removes the official client CSS helper.
Native output is retained, including its existing client CSS helper difference.
The exact embedded-script replay finishes at Elk DOM 10717/13561 and SSR
9218/25007, with three official errors and no native errors. The existing SSR
addition budget of 9201 still fails.

The unchanged npmX Inspector environment resolves the repository's public
Vue 3.6.0-beta.10 and finishes at DOM 17037/22952 and SSR 14498/36510, with no
compiler errors. Its separately prepared Vue 3.5.39 graph finishes at DOM
17029/22944 and the same SSR counts; these providers are kept distinct. Both
retain all 220 native code/error/warning packets and source/provider hashes.
The pinned npmX corpus contains no CSS variables, so its changes come only
from the scoped-slot correction.

This local adapter replay is not a source-built CLI acceptance claim. Linux
validation of the rebuilt CLI and unchanged application gates remains pending
in Actions at the final source revision.
