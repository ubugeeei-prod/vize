# VueFes SSR style dependencies (#6880)

[Compiler fix-history issue #6880](https://github.com/ubugeeei-prod/vize/issues/6880)
owns this regression fixture. The correction keeps each metadata-bearing SFC
style block as a real virtual CSS import in SSR output, as in client output.
Frameworks need these imports to discover component styles through Vite's SSR
module graph. Previously plain CSS without preprocessors, CSS Modules or
`@apply` lost those edges in SSR.

The production change is the style-import predicate in
`npm/builder/vite/src/utils/index.ts`. Scope attributes, native code generation,
stylesheet contents, inline CSS fallback and the original VRT budgets keep
their existing contracts. This adds no compiler pipeline stage or CSS reparse.

## Actual failure and cause

The original full VueFes VRT failed on source
`605429c660b328bb03bec5bbfef15961e59fcaf0`:
[run 38111801880, job 114388841982](https://github.com/ubugeeei-prod/vize/actions/runs/38111801880/job/114388841982).
Desktop dev-home had a difference ratio of 0.1235105465 against its original
0.002 budget. Reference and candidate heights were 17,968 and 17,870 pixels.

The bounded supplementary runtime capture retained the original assertion:
[run 38115253920, job 114398771782](https://github.com/ubugeeei-prod/vize/actions/runs/38115253920/job/114398771782).
All three original retries failed at 0.12349611114203028; the serial workload
reported one failure and 37 cases not run. This run is diagnostic evidence,
not acceptance, merge or release proof.

All fourteen actual candidate heading roots have their correct
`data-v-98cff51b` attribute. Their complete client component module has its
correct scoped-style import and scope assignment. Missing native scope
emission is therefore disproved. The scoped `h2[data-v-…]` rule and global
`:root h2` rule both have specificity `(0, 1, 1)`. The reference places global
CSS at sheet 0 and heading CSS at sheet 6, producing 20px text with 34px line
height. The candidate places heading CSS at sheet 2 and global CSS at sheet 9,
producing 18px text with 27px line height. Fourteen 7px differences explain the
98px document height difference. The complete DOM, CSSOM, stylesheet bodies,
module responses and all original screenshots are retained in artifact
11693093826, SHA256
`0d096e86f6f556e2b2493328782b7050b59223ef1822e6f7152a8bee7c1ee546`.

[Nuxt 4.0.3's manifest discovery](https://github.com/nuxt/nuxt/blob/v4.0.3/packages/vite/src/vite-node.ts#L61-L93)
enumerates CSS from the SSR module graph in insertion order.
[Official plugin-vue 6.0.0](https://github.com/vitejs/vite-plugin-vue/blob/plugin-vue%406.0.0/packages/plugin-vue/src/main.ts)
keeps SFC style imports in SSR. The missing plain-style dependency is the
bounded producer correction; only the unchanged original application VRT can
establish whether it repairs VueFes's full cascade.

## Durable regression and qualification

The [whole-source corpus](../../../tests/_fixtures/differential/compiler/vuefes-heading-cascade/corpus.json)
preserves the complete 1,359-byte `VFHeading.vue` and 5,959-byte `main.css` from
read-only VueFes commit `8eff3cd7dd79a7f47bbf2b78202b9aa2070ec5a6`.
Both SHA256 and upstream Git blob identities are checked. No style block,
template branch, rule or original import is removed. The runtime fixture uses
the same pinned upstream asset closure, with only its transport entries authored
locally. Upstream receives no write.

The native metadata test covers the original full SFC, a plain-style control,
and the full original plus a separately declared second style block. Both SSR
and client imports and native source maps are checked. The actual Vite runtime
differential uses exact official Vue, compiler and renderer 3.6.0-alpha.2;
Vite 7.1.5 and official plugin-vue 6.0.0 are explicit isolated controls. The
fixture's private npm configuration allows this historical prerelease through
the official plugin's stable-only peer range, while runtime version assertions
enforce all actual Vue packages. This does not change product dependencies.

Official plugin-vue's public `componentIdGenerator` deliberately uses the
native adapter's absolute-filename hash, so complete SSR, cold DOM and hydrated
DOM can be compared without normalizing outputs or scope attributes. The
fixture retains actual complete module graphs, outputs, stylesheets and browser
observations. A separate negative producer removes only the native SSR CSS
import and must fail the dependency oracle. A separate CSS-order counterexample
moves the complete original global stylesheet after heading styles and must
reproduce 18px/27px; the normal observations remain retained.

The source qualifier reuses the existing original `full:vrt:vuefes` row,
including all 38 ordered cases, its 15-minute workload deadline and its original
budgets. The native build is reused for the differential and existing Vite
contracts. The original VRT remains uninstrumented. Whole failure artifacts,
complete generated declarations and declaration differences are uploaded.

## Remaining acceptance

The first source qualifier on `e23560400b7642e4925c98e6da513b628ce64709`
([run 38117641371](https://github.com/ubugeeei-prod/vize/actions/runs/38117641371))
failed the unchanged dev-home VRT at 0.1235105465271594 on all three retries;
37 later cases were not run. Retaining the SSR dependency alone did not repair
the application cascade. The native metadata test also exposed its own invalid
nonnull source-map assumption: SSR maps are optional. The regression checks
that imports preserve the compiler's actual optional map contract and explicitly
selects the non-Vapor compiler.

The whole official/native fixture retained passing cold DOM and hydration
observations, but then its response-body observer rejected a redirect. It also
retained native direct stylesheet responses with HTTP 404 and raw native SSR
with eight duplicated scope attributes. None is excused by the DOM observations.
Whole source artifact 11694287884 has SHA256
`75a5082e87c8303d19160befa66193f7a1bfa9ecda1110826f1ae7e37c4d9c2c`;
the separate original VRT artifact 11694228045 has SHA256
`aa9517114adde002f444095832cd08af607cf277ccfe10acbb1bad16b9e43613`.

The dedicated source qualifier is manually dispatched against the composed
Stack's literal head once its dependent source fixes are sealed. This avoids
repeating the full native build and 38-case workload for incomplete intermediate
layers. Existing shared App E2E and protected checks remain required.

The actual original-H diagnostic native binary and Vue runtime are distinct
from the earlier published 0.443.0 Darwin control. Its mistaken rc.11 control
is superseded by the exact alpha.2 control and is not acceptance evidence.
The diagnostic completeness gate remains red for 34 HTTP 304 revalidations
without protocol bodies, despite retained earlier 200 bodies and complete
CSSOM. Its strict post-build source gate remains red for the complete generated
`npm/native/index.d.ts` delta, including documented type changes. These results
are preserved and receive no exemption.

Source Actions, unchanged full application VRT, actual protected merge and
release consumer verification remain required. Neither this decision nor a
fixture pass closes the compiler fix-history gate or proves a release.
