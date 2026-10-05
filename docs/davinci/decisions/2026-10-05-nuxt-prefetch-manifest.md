# Original Nuxt SPA manifest and prefetch ownership

Paired [P0 #7999 decision](https://github.com/ubugeeei-prod/vize/issues/7999#issuecomment-5989320594).
The reported compiler-on manifest retains `pages/foo.vue?vue&vize`, so Nuxt's
plain-source page/global-component filters keep route chunks in entry prefetches.
The original issue body and source hashes live in
`tests/_fixtures/differential/compiler/nuxt-prefetch-manifest/`.

## Producer and repair authority

The pinned [Nuxt 4.5.2 core](https://github.com/nuxt/nuxt/blob/v4.5.2/packages/nuxt/src/core/nuxt.ts)
installs user modules before core private modules. The original
[pages filter](https://github.com/nuxt/nuxt/blob/v4.5.2/packages/nuxt/src/pages/module.ts)
and [global-component filter](https://github.com/nuxt/nuxt/blob/v4.5.2/packages/nuxt/src/components/module.ts)
compare `dynamicImports` with paths relative to the original source directory.
The [manifest writer](https://github.com/nuxt/nuxt/blob/v4.5.2/packages/vite/src/manifest.ts)
normalizes the Vite manifest, invokes `build:manifest`, then precomputes dependencies.
The original [SSR style producer](https://github.com/nuxt/nuxt/blob/v4.5.2/packages/vite/src/plugins/ssr-styles.ts)
retains its plain emitted style keys and subsequent cleanup.

The existing enabled-Vize setup registers a normal production `build:manifest`
hook before those core consumers. Only a main-request key ending exactly in
`.vue?vue&vize` with the identical `src` establishes ownership. The complete
original row map determines the new plain key, `src`, `imports` and
`dynamicImports`; no reference is rewritten without its actual manifest row.
Payload objects retain identity, output files/CSS/assets remain untouched,
foreign/extra/raw/style/legacy requests stay byte exact, and source-key collisions
fail before any mutation. Development module IDs, host/compiler-off paths,
native compilation and all instruction ceilings retain their existing contracts.

Three pure laws compare complete original graphs, preserve unrelated request
bytes and require all-or-nothing collision refusal. The additional corpus and
ordered-prefetch laws reject changed original sources, unknown resource owners,
missing graph rows, omitted/reordered links, changed attributes and changed
dependency ownership. Default native custody still requires both client and SSR
calls; the original `ssr: false` case explicitly qualifies only client calls.

## Original sources and automatic execution

The supplied config and reports page retain their complete issue code blocks.
The supplied app/index inline sources retain their text with a terminal LF.
The issue specifies any scoped ReportTable; the complete new scoped component
is independently authored and explicitly marked in the corpus authority.
Original routes remain `/` and `/reports`.

The existing automatic Nuxt 3 job retains its actual source-native build,
whole original Nuxt 3 inputs/client/SSR returns and Chromium assertions.
It additionally installs the unchanged critical-CSS dependency lock and runs
three cold original SPA generates: compiler-off stock, genuinely packed old Nuxt
integration source `4e4c8c977d7052edbb6a2b42808ddc6e7b59f3bf`, and current source.
Both Vize arms use the same authenticated current source-built NAPI binary and
Vite adapter. The literal old source is fetched for shallow checkouts, archived,
packed and hashed; it is never simulated by disabling the new production hook.
No original style, critical-CSS or Nuxt 3 fixture/lock is rewritten.

Every arm retains full original fixture hashes, native inputs/returns/code/maps,
client manifest/precomputed modules, prerender HTML and browser DOMs, all link
attributes and raw physical requests/responses/errors. The prior arm must retain
both query-keyed page imports and actually prefetch their JS/scoped CSS on home.
Stock/current must omit those same page assets on home, load them on the reports
route and show the original red scoped paragraph without errors.

The complete entry dynamic-import vector must equal stock. Every ordered
prefetch resource must join its actual emitted filename to all contextual
manifest dependency paths and unchanged link attributes. Current/stock vectors
must match strictly. Raw hashed filenames remain retained separately; this
qualifies contextual prefetch ownership and does not claim compiled JS byte
equality across compilers. Unjoined resources or changed complete vectors fail.
Failed arms retain raw evidence before qualification; launch and browser failures
close the HTTP server and browser so the diagnostic remains finite.

The automatic dependency cohort is Nuxt 4.5.2/Vue 3.5.43. The reporter's Vue
3.5.42 and large-app 398 versus 81 files/359 versus 40 prefetches remain historical
reports. They are not newly measured and their exact package cohort is not
claimed as executed. This original SPA claims no SSR, native migration or speed
credit. Future changes to the actual runner/corpus/fixture/custody paths select
the same automatic source-native job; no manual campaign or new workflow exists.

## Scope and pending gates

Independent production review is clear; eight local pure laws and configured
format/lint pass. Actual current-head hosted source/native/Nuxt/Chromium execution,
the unchanged original differential contracts, protected full suites and all104
instruction ceilings, signed actual merge and the next supported release remain
pending. A draft PR may publish only after the bounded source/root review.

Read-only #7983 diagnosis is separate: `lint/emitter.ts` resolves plan globs
against the Nuxt root, then makes them relative to the `.nuxt` config directory;
`generation.ts` supplies that exact root/config pair. The resulting `../` ignore
and override patterns conflict with the reported Oxlint namespace. This manifest
change does not modify that boundary or claim the startup failure repaired.

## Previous CSS delivery sealed

The separately owned #7875/#7826 slice actually merged as signed
`4e4c8c977d7052edbb6a2b42808ddc6e7b59f3bf` at 2026-10-05T05:13:31Z;
#7826 closed one second later. [Protected Check37265659999](https://github.com/ubugeeei-prod/vize/actions/runs/37265659999)
is terminal success with full Rust/four workers, full300 historic inputs,
the strict 23 historical + one qualified current + one internal capture,
27 public API cases/81 passes and nine CLI cases/36 phases. Authentic three-run
100+4 instruction artifacts pass the unchanged ceilings. The historical184-byte
mismatch remains distinct from the whole185-byte current reference.
The bounded receipt is `/tmp/vize-7875-4e4c-protected-receipt.json`, paired with
the [terminal issue receipt](https://github.com/ubugeeei-prod/vize/issues/7826#issuecomment-5989021683).
Publication/installed-package proof remains the release owner's next supported
candidate. Private #7866 remains unpublished; it contributes no #7999 evidence.

## Original fixture formatting integration

[First-source correction](https://github.com/ubugeeei-prod/vize/issues/7999#issuecomment-5989614560)
records Check37274463184/check-js111648329654 on `8fa93b1b`: the repository
formatter would change the exact original config and scoped-component inputs.
Preserve all six fixture hashes and every production/capture/oracle byte, and
add only this original SPA fixture directory to the established formatter-sensitive
input list. Fixture lint remains enabled; no global bypass or instruction waiver
is introduced. Both original L1/L3 Vue inventories remain exactly449 files, and
the check snapshot declaration law passes without fabricated registrations.
The earlier successful Nuxt source-native/generate/Chromium steps remain distinct
from artifact qualification and cannot transfer to the successor head. Fresh
ordinary source Actions, authentic full packets and protected suites are required.

The same source tooling3 job111648482245 separately rejects growth of the existing
Nuxt entry from639 to648 lines. Move-only `4a20cc0b09` extracts the unchanged
`isPlainRecord`/`mergePlainRecords` statements into24-line `module-records.ts`.
The entry shrinks to624 lines; reviewed manifest production and all existing call
behavior remain intact. The350-line ratchet is unchanged, with no waiver.

The first artifact11329991020 also follows the temporary prior-pack dependency
symlink during upload. Unlink only that borrowed link in the pack `finally`; keep
all source/archive/dist/hash/raw/native packets and original dependency bytes.
This finite cleanup requires fresh execution and adds no runtime qualification.
