# Vapor SFC CSS variable runtime ownership

Issue [#7887](https://github.com/ubugeeei-prod/vize/issues/7887).

At audited main `ef506012169d228982ded40d893ac2115a3e58a7`, inline SFC
assembly always imports and invokes VDOM `useCssVars`. A real Vapor component
has a different instance owner, so no root custom properties are installed.
Select `useVaporCssVars` using the already effective client renderer flag in
both preamble and setup emission. Preserve the getter's original variable
names, expression transformation, scheduler and existing unref import rule.
No new stage, serialization or public option is introduced. Standard VDOM
and the explicitly diagnosed existing Vapor SSR fallback keep `useCssVars`.

The complete reporter SFC is retained exactly: 210 bytes, SHA256
`e34934a3bfc2518422e5c8e2ab6ee375a6fc65b89af10a457b2ade47becccfd7`.
The original issue body and environment remain distinct from test versions.
The repository pins official Vue/compiler-sfc/runtime-vapor `3.6.0-rc.9`;
the reporter compared `3.6.0-rc.10`. Primary rc9 compiler CSS variable helper
selection is blob `d7be1b149b1e7a4df90166e3e7a902a90d0702b8`, and the actual
Vapor helper is blob `be32fa9d46cede3caaed346c04287f5a7ee59b55`.
The real SSR fallback oracle uses the locked stable Vue `3.5.35`.

A fixed regression compiles the original attribute-selected SFC, the ordinary
setup equivalent selected through the public Vapor flag, a typed equivalent,
and a standard VDOM control in development and production, client and SSR:
16 complete result pairs with source maps off/on. Mapping must change no code,
CSS, bindings, artifacts or diagnostics and retain the original source name.
Each environment launches a fresh process with its actual pinned runtime.
Full official and source-built components mount unchanged after normal module
import/TypeScript handling. No CSS-helper stub or injected instance is used.
The mounted root must own the exact custom property referenced by its complete
CSS, update red to blue on click, retain identity, remove all DOM on unmount
and emit no warning/error. Whole normalized DOM traces and values must equal
both frozen expected behavior and the independent official compiler.

Production variable names are backend-specific. Normalize only the one real
CSS declaration's authenticated property name for semantic comparison and
retain each complete unmodified code/CSS, source map, raw HTML and observation.
The eight SSR pairs use actual stable server rendering and retain the exact
`VAPOR_SSR_FALLBACK` diagnostic on the six Vapor requests; this gives no direct
Vapor SSR, hydration or Chromium credit. The full passing packet is stored
beside each actual nextest JUnit artifact, without extra compiler execution.

TODO: exact-head source Actions and independent review, protected full suites
and unchanged 104 instruction ceilings, actual signed main and recognized
reporter trailer, then an authenticated release. This existing-product repair
adds no native-stage/default migration, benchmark ranking or speed claim.
Whole original 17 CSS and 33 runtime benchmark qualification remains separate
under #7856; no historical failed snapshot is called current-source failure.

Paired source decision: [#7887 comment](https://github.com/ubugeeei-prod/vize/issues/7887#issuecomment-5987533072).

Before the first source Rust execution, reading the locked stable Vue `stringifyStyle` confirms SSR serializes custom properties as `key:value;`, whereas live DOM setters serialize `key: value;`. Correct only the separately authored frozen SSR formatting expectation and its authenticated digest. Preserve full raw HTML and the complete semantic oracle; no expected value is derived from Vize output and no old-head acceptance transfers.

Paired oracle correction: [#7887 comment](https://github.com/ubugeeei-prod/vize/issues/7887#issuecomment-5987578581).

## First source runtime precondition and raw failure custody

Source `b3058ed4ef` Check37259527439 passes Clippy, JS/tooling/browser,
Nuxt and three Rust shards. Rust3 job111605523950 fails the new oracle:
the first official rc9 Vapor mount owns and updates its root CSS property,
but emits `ReferenceError: MutationObserver is not defined`. The happy-dom
global bridge omitted the helper's real Window API. The expected diagnostics
remain empty, and this failure precedes first Vize component evaluation;
no source-built mounted acceptance follows. Original authenticated raw log
SHA256 `0215895d16df5de388d15920b66bda138adc341fd2573dbb10fcd7cf9faf7636`
and failed JUnit artifact11324685344 are preserved.

Register only the actual `window.MutationObserver`, without a stub or helper
replacement. Preserve every original source, frozen CSS value/tree and the
three production files. Retain full original input/stdout/stderr bytes/exit
before Rust assertions, and preserve complete/partial official/actual rows
and raw modules even on failure. Successful complete16 pairs remain mandatory.
No extra compilation or best-of selection is introduced.

The current-main projection is clean; historical prospective queue prefix
`95a1c482b7` conflicts only at the shared canonical paragraph339. Relocate
this complete decision/failure history beside existing Rust source wiring,
preserving all incoming clauses without importing unmerged production/docs.
Fresh successor source Actions and protected gates remain required; the
failed head's separate passing Nuxt lanes confer no CSS runtime acceptance.

Paired runtime precondition/custody correction: [#7887 comment](https://github.com/ubugeeei-prod/vize/issues/7887#issuecomment-5987717993).

## Symmetric SFC adapter metadata after actual runtime execution

Source `30a8ef22a5` Check37260673399 executes the real MutationObserver and records six successful development pairs: original/flag/typed clients and their diagnosed SSR fallbacks. The seventh ordinary VDOM control fails only its missing scoped DOM attribute; its CSS variable installs and updates correctly. The independent reference already attaches component `__scopeId`, while raw Rust inline output delegates that metadata to the existing Vite adapter (`npm/builder/vite/src/utils/index.ts`, audited main `ef506012169d228982ded40d893ac2115a3e58a7`, lines191–220). Retained full process-development input/stdout/stderr/exit and official failed JUnit artifact11324936610 establish this exact asymmetry. Original log SHA256 is `717ee6f3d86444fe5af8c1c0cac16502ea4caa065b382250434fbe5117d873ad`; the archive digest is `1e1f598e0758a6e25635a609a97ad9afb3cad694303d10cac0c504af053153f7`. Production-mode pairs were not reached and overall acceptance remains incomplete.

Apply the existing ordinary SFC scope metadata attachment equally to both evaluated default components and record the actual attached scope ID. Preserve every original generated code/CSS/map, expected complete tree/value and all three production files; no CSS helper, instance or DOM attribute is synthesized. Fresh exact-head Actions must execute all16 pairs. Client rc9 runtimes are explicitly built for development/production; stable SSR uses its ambient locked runtime with corresponding compiler options, not an independently attested stable-SSR runtime-mode matrix. Maps attest whole-result additivity/source name, not decoded mapping coordinates.

Paired symmetric adapter correction: [#7887 comment](https://github.com/ubugeeei-prod/vize/issues/7887#issuecomment-5987993931).

## Actual-main canonical composition preservation

Reviewed scope-metadata successor `123b3a8a42` cannot start configured source Check because actual main `7cc79441ab35a0add0f4f90d4bbab0d4bef52e52` now records the delivered page-meta ownership history on the same canonical paragraph324. The only merge conflict is documentation; no compiler/runtime/fixture conflict or CI failure follows. Preserve every complete incoming main paragraph and the complete CSS-variable decision/failure clause on the existing350 lines. All owned non-doc source, production, original/expected and review hashes remain unchanged. Fresh configured exact-head Actions and all16 runtime comparisons remain mandatory; this composition repair transfers no earlier failed-source or future protected credit.

A content-only canonical union is insufficient for Git's line-based three-way merge on this same paragraph. Replay the five existing commits genuinely onto actual main `7cc79441ab35a0add0f4f90d4bbab0d4bef52e52`, retaining every incoming canonical line and the complete owned history. All eleven owned production/Rust/helper/corpus files match the reviewed source exactly, and all five original author/email/date/full-message/reporter-trailer records remain exact. The actual main is now a source ancestor; fresh configured Actions, not prose-only composition or historical gates, must qualify this replay.
