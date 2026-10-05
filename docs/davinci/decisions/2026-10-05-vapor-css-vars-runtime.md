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
