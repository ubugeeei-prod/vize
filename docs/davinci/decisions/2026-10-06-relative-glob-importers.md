# Preserve original SFC importers for relative globs

Issue: [#7936](https://github.com/ubugeeei-prod/vize/issues/7936).

The full original report has nine code blocks, including both Vite root
configurations, the complete Plain/Alpha sources and the original commands.
The reporter is verified public GitHub user `ubugeeei`, ID `71201308`; the
meaningful implementation and PR retain the recognized noreply co-author.
Original package dependencies remain pinned separately from the candidate
adapter/source binding. No original source, pattern or expected key is changed.

## Cause and narrow change

The original #884 rewrite made relative glob literals root-absolute so Vite
could process null-prefixed virtual modules. Current normal SFC modules instead
have file-based IDs, including `/file.vue?vue&vize`. Vite resolves their globs
against the actual importer and retains relative keys. Rewriting those literals
still changes the keys, and a filesystem-absolute path outside the Vite root is
interpreted as a root-relative pattern rather than the intended external file.

Only genuinely null-prefixed compatibility loads continue to use the existing
native rewrite. Physical and plugin-visible SFC loads pass original patterns and
options to the existing Vite glob transform. The public Rust helper, its old
whole-output laws, null-ID behavior, unrelated load rewrites, source-map editing,
module registration and HMR remain unchanged. No pipeline stage, AST pass, IPC,
new public API or instruction ceiling is added.

The [pinned Vite 8.2.2 source](https://github.com/vitejs/vite/blob/v8.2.2/packages/vite/src/node/plugins/importMetaGlob.ts)
distinguishes file and virtual importers and derives relative keys from the
file importer directory. This change restores that existing importer policy.

## Whole fixture and runtime contract

`tests/_fixtures/differential/compiler/vite-relative-glob` retains every original
block with byte/SHA pins. The explicitly reported src-index HTML substitution is
recorded as a derivative. Separate authored Lookup/Options controls cover exact
eager/default lookup, array/negation, template literals, base, raw-query contents
and no-match behavior. Whole HTML references derive from literal authored keys,
original Alpha bytes, Vue text escaping and the templates. They are frozen
before Actions and are never copied from current execution.

The existing Nuxt module workflow installs one new locked reproduction with the
reported Vite 8.2.2/plugin-vue 6.0.8/Vue 3.5.43/package versions. Existing Nuxt
3.19.3 and 4.5.2 locks, both using Vite 8.3.1/plugin-vue 6.0.9/Vue 3.5.43, stay
unchanged. Each cohort uses original project-root and src-root layouts and
compares both the real stock adapter and the source candidate against complete
fixed references. Actual client builds are mounted in Chromium; SSR bundles
execute through Vue's renderer. Full bundle/load/native/process/browser outputs
and exact original inputs are retained, including on failure. Source native
custody uses the existing scoped physical-addon receipt and preload.

Nuxt coverage means genuine `loadNuxt`/module setup followed by actual Vite
client/SSR builds and full component runtime outputs. It does not claim full
Nitro generation, browser hydration, performance, native migration or retirement
of any unfinished fix-history issue. Existing full Nuxt laws remain unchanged.

## Qualification and delivery

At preparation, original byte controls pass but Rust/native/build/browser runtime
is unexecuted locally. Fresh exact-head Actions, complete protected Rust/tooling
corpora and all 104 unchanged performance caps are required. Independent squash
auto-merge starts only after current source checks and whole-prefix composition
are healthy. Track actual signed merge, reporter trailer and next root-owned
public release separately. Earlier #8050/#8042 receipts and source refs remain
preserved; the original conflicted checkout and shared caches are untouched.

## First source failure and retained-query correction

Source `7596068` / hosted merge `c0d8e6c` failed the new Options raw-content
control in run [37351433010](https://github.com/ubugeeei-prod/vize/actions/runs/37351433010).
Stock rendered the literal escaped original Alpha source; Vize rendered `{}`.
The preceding project-root Plain/Lookup client and SSR whole results matched,
but the failed aggregate does not qualify all 72 planned cells. Raw failure and
all original authored references remain retained; no expected output is changed.

[#8062](https://github.com/ubugeeei-prod/vize/issues/8062) records the cause:
`resolveRelativeVueSfcImport` converted an effective `?raw` request into a
component ID. The same helper now preserves Vite's complete resolved envelope
for the four established `raw`, `url`, `worker`, `sharedworker` query flags,
matching existing general resolve/load policy. The original raw-glob reference
is retained. Additional whole-envelope controls cover both graphs, dependency
scans and an asset query introduced by an upstream resolver. Normal relative
components, original override controls and all Rust/#884 laws remain unchanged.

Source Check [37351433846](https://github.com/ubugeeei-prod/vize/actions/runs/37351433846)
also rejected growth of the grandfathered `load.ts` from 465 to 477 lines.
Condensing only existing JSX-HMR explanatory comments retains their meaning and
returns that file to its existing 465-line ceiling; no budget or gate is raised.
Map metadata is retained by the runtime probe, but these fixtures do not request
build source maps or claim new segment/stock-map parity. Fresh successor source,
complete 72-cell runtime and protected acceptance remain required.
