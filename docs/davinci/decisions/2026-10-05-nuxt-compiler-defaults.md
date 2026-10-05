# Nuxt inherited compiler defaults (#7959)

The original [report](https://github.com/ubugeeei-prod/vize/issues/7959)
uses Nuxt 4.5.2, Vue 3.5.43 and Vize 0.432.0. Its five complete code blocks
are immutable compiler corpus inputs. `LooseRow` is the reported tight-to-loose
name substitution, explicitly recorded as a derivative rather than an original
code block. The reporter is verified GitHub user `ubugeeei` (71201308).

## Decision

Nuxt's own `vue.compilerOptions.whitespace` is an inherited default. Authored
Vize top-level compiler values, matching ordered entries, explicit module
compiler values and explicit template compiler values retain their authority.
The existing Vite `vize` function gains an additive optional second argument
containing only `whitespace`. Its order is inherited default < project compiler
< matching entries < explicit plugin value; the established nested template
compiler option still takes precedence during compile-option projection.

Only native Nuxt plugin registration separates the inherited value from the
resolved options. Inspector options and unsupported/legacy host fallback retain
their existing complete resolved objects. Compiler exclusion, on-demand scanning,
Nuxt page metadata, source-relative SSR registration, Vapor and HMR are unchanged.
The module's cold native-plugin setup moves into a small helper to preserve the
existing entry-file length ratchet. No public config schema field is added.

## Authored reference and evidence plan

The corpus pins every original byte, including the unused diagnostic `check.mjs`
script. Eight controls cover original forwarding off/on, an unconfigured inherited
default, the unconfigured compiler default, top-level overrides, entry overrides,
explicit module and nested template options, and a condensed Nuxt default below
an authored preserving project. Existing parser `preserve_whitespace` trims edge
runs and reduces the middle run to one ASCII space; condense removes the original
newline gap. These source-derived rules independently author every entire
`<main>` HTML result. No current generated output is recorded as an oracle.

The existing automatic Nuxt 3 source-native workflow reuses its unchanged installed
Nuxt 3.19.3 and Nuxt 4.5.2 / Vue 3.5.43 dependency cohorts. A fresh process loads
the whole original Nuxt config for each control. The actual module-supplied plugins
then run real Vite client and SSR builds of all three original/derived SFCs. Every
complete public load result, bundle object and native return must equal a separate
unchanged explicit-whitespace public Vite control selected by the frozen matrix.
Both emitted bundles are executed by the genuine Vue server renderer and compared
to the full authored HTML and complete SSR module-ownership set. Actual raw process
logs, all native calls/returns, load envelopes, bundles and rendering are retained
before assertions; no installed dependency tree or original source is modified.

This proves Nuxt setup -> Vite compilation -> Vue rendering, and does not claim a
full Nuxt/Nitro generate, browser hydration, startup speedup or native migration.
Existing full Nuxt 3 builds, SSR/hydration, Nuxt 4 SPA manifests, source-native lint
controls, legacy corpora and all instruction ceilings remain required and unchanged.

## Qualification and remaining work

Original-source/hash and authored-matrix pure tests passed locally. The source
option test cannot run in this light checkout because the package-local native
binding is absent; no install or local native build was attempted. Whole compiled
Nuxt 3/4 execution remains unknown until exact-head automatic Actions. Protected
104 gates, actual signed merge, final reporter attribution and publication remain
pending. First-v0.433 admission HOLD remains active: this source PR has no auto
merge or queue request, and neither #7959 nor any fix-history issue is closed.

## First source qualification correction

Source `5bb75baf` is not accepted: [the paired failure decision](https://github.com/ubugeeei-prod/vize/issues/7959#issuecomment-5997032650)
retains Nuxt3 run 37327024046 and Check run 37327025125. The native compiler,
option controls and all existing Nuxt3/4 builds/SPA/source-lint proofs passed,
but the new probe stopped before Vite compilation on an authored trailing-separator
assumption about Nuxt3's `srcDir`. The original report specifies no such policy.
Compare the resolved owned directory while retaining the unaltered full actual
Nuxt options; remove the independently rejected unused import. Production bytes,
all original inputs, frozen matrix, whole compilation/render references and caps
remain unchanged. Exact successor Actions must qualify them afresh; the first
failed logs and small selected artifact members remain historical evidence, with
no full-archive digest, current compiled acceptance or queue/release claim.
