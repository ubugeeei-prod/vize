# Original App check and prepared-provider baseline audit

Related: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879), and
[the generic-call decision](./2026-10-11-6879-generic-call-inference.md).
No App diagnostic golden is changed by this audit.

## Source and original qualification

The retained composed [run 38138961313](https://github.com/ubugeeei-prod/vize/actions/runs/38138961313)
authenticates source `8170fd3e28885b0de430fd9cd8bbece666cbce18` through
`source.sha` and all 28,140 tracked entries in `whole-source-index.txt`.
That source descends from `1a0d0cbb0070c861e514864e1727b62b1192087d`, but also
contains the separate Nuxt installation/preparation recipe. It is not an
exact-head qualification of another source or provider setup. The run failed.

The original complete App check passed [run 35979422921](https://github.com/ubugeeei-prod/vize/actions/runs/35979422921/job/107567757745)
at `b5d7298c36bc058664c89a6c289b8021756c1b15`, and independently
[run 35981081660](https://github.com/ubugeeei-prod/vize/actions/runs/35981081660/job/107573123511)
at `f0b776df7275011ab59cfe59bb455a5c684ad6c9`. Both explicitly report
`Snapshot matched` for all three projects. Their source pins and golden blobs
equal those at `8170fd3e` and observed main `d55b8238f89bab4418a1a961bd8db1c84911cb42`.

| Project | Authored source pin                        | Original golden Git blob                   |
| ------- | ------------------------------------------ | ------------------------------------------ |
| elk     | `8aa2b4650760eeba4ea1fdd9dc18d8a88ca5d7f0` | `6e329a716a454f9457ceea445452a62d2848c021` |
| nuxt-ui | `3dbca020969582d577c7df9698dc8cd484f76dc1` | `6aa6be3bdb9702e59ea5f5ea9d87549c8f515856` |
| reka-ui | `c23e0ed8a38c66cd4f9350e7a46c6b502a8491c2` | `211c75eaeb898c3f5d61c4fc5bb1bbe063c7d30e` |

The historical logs also warn that generated `.nuxt` types are absent and
Nuxt auto-imports fall back to `any`, missing some type errors. Historical
green proves complete snapshot equality under that setup; it does not prove
the correctness of newly prepared providers.

## Full report comparison

Every original file object was compared by filename and its entire ordered
diagnostic vector. No original reported file disappears. The following counts
describe reported membership, not independently proved native root membership:
the snapshot formatter removes `programs`, and `fileCount` counts reported files.

| Project | Report files before / after | Identical original file objects | Changed original file objects | Errors before / after |
| ------- | --------------------------- | ------------------------------- | ----------------------------- | --------------------- |
| elk     | 262 / 279                   | 123                             | 139                           | 286 / 3               |
| nuxt-ui | 231 / 398                   | 16                              | 215                           | 1677 / 551            |
| reka-ui | 913 / 928                   | 157                             | 756                           | 3113 / 280            |

Elk adds seventeen authored TypeScript files; ReKa adds fifteen authored
TypeScript files. Without historical whole native programs, their cause cannot
be attributed solely to a particular scanner or configuration change. Nuxt UI
adds 166 generated `.nuxt` files and one installed `@nuxt/icon` JavaScript file,
whose preparation/provider provenance is retained. Added files are diagnostic
empty except six ReKa module-resolution rows in four files.

On original file objects, Elk removes 285 diagnostics and adds two; Nuxt UI
removes 1135 and adds nine; ReKa removes 2845 and adds six. These are complete
diagnostic multiset differences; the original ordered vectors remain authoritative.
The respective removed TS2307 counts are 122, 943 and 1825. These reductions
include newly available providers and aliases; counters alone cannot distinguish
correct resolution, changed configuration, or lost checking.

## Provider and configuration observations

Nuxt preparation was introduced on this composed branch by
`f68c124eab94f1f9e1e479cbaa349f399fc4863f`. The recipe invokes
`pnpm@10 install --no-frozen-lockfile --ignore-scripts --prefer-offline`, then
Elk `nuxt prepare` or Nuxt UI `dev:prepare`. Receipts report all authored files
preserved, no source mutations and no lock changes, but explicitly classify
installation as unlocked. This preparation is absent from the observed main.

Observed providers are Elk Nuxt 4.4.7 / Vue 3.5.30 / TypeScript 5.9.3, and Nuxt
UI Nuxt 4.5.2 / Vue 3.5.41 / TypeScript 6.0.3 / `@nuxt/icon` 2.5.0. Original
manifest/declaration/config bytes and hashes are retained in each preparation
receipt. Installed declaration custody does not establish native dependency reads.

Elk reports `.nuxt/tsconfig.app.json` with strict Bundler/preserve options and
48 alias entries. Nuxt UI reports root `tsconfig.json` with strict Bundler/preserve,
`noUncheckedIndexedAccess` and 44 alias entries. ReKa reports
`packages/core/tsconfig.app.json`; its authored extension
`@vue/tsconfig/tsconfig.dom.json` fails resolution. All three App receipts mark
native exec-time selected config/options and dependency-read identities unproved;
their guessed generated-config candidate is missing.

The independent pinned ReKa native packets retain the actual nested generated
config, native project, options and matching roots. They still observe no
`@vue/tsconfig` provider at any recorded scope. `strict`, `module` and
`moduleResolution` are absent from those actual options. Absence is not a claim
that native `strict` is false. The explicitly selected pinned project is also
not presented as equivalent to the App CLI's unspecified config selection.

## Confirmed gaps and focused correction candidates

All fifteen existing injected TS2322 controls pass. Each checks the entire
injected-file vector: clean is empty, broken has exactly its authored TS2322,
and repaired equals clean. They execute 45 commands, each reporting one
standalone file; ReKa's injection is under `packages/`, outside `packages/core`.
There are no complete broken/fixed native config/provider packets for these
controls, and Ant Design Vue is explicitly skipped. This evidence cannot
qualify every dropped whole-project diagnostic or generic refusal.

All six pinned PinInput cases, including the genuine wrong-number source,
return the same 27 TS2307, four TS7006 and two TS7053 rows, with no TS2322.
This is a confirmed missing generic refusal, not a successful positive case.
The requested native roots/options match the retained packets, and all complete
PinInput/Tree source/code/map blobs validate. Fixing directory-barrel resolution
and authenticating the configured provider remain necessary; no verdict is waived.

Nuxt UI newly reports TS2307 on the relative Vue imports in
`src/runtime/composables/useToast.ts`, `useFormField.ts` and `useContentSearch.ts`.
The corresponding `Toast.vue`, `FormField.vue` and `content/ContentSearch.vue`
all occur in both the reported program and full report. Their complete authored
bytes are retained. This is a concrete mixed TypeScript/Vue resolution candidate
for #3984: reproduce through the ordinary public batch factory and actual native
client, retaining the chosen config, whole program and complete raw/mapped vectors.
The regression must include a genuinely missing neighbor and a repaired typed
import, rather than suppressing module diagnostics.

Nuxt UI's new InputTags and PinInput generic errors and CommandPalette callback
error require a configured reference-oracle comparison before they are classified
as false positives. Elk's new `UserLogin.id` rows likewise require the preserved
authored type and actual provider. No blanket diagnostic deletion is justified.

## Baseline migration requirements

Before proposing new prepared-provider goldens, preserve these original blobs
and raw failures, bind the same source pins to complete authored file hashes,
and compare the same product source with preparation disabled/enabled. Capture
the actual native selected config/options/root membership and every provider
identity used by each program; repair missing ReKa config authority first.
For every changed file vector, compare a genuinely configured reference oracle
and retain both false-positive and false-negative controls at those original
program scopes. Keep all original App cases and fifteen negative controls.

Only a reviewed complete per-file classification plus fresh exact-source Actions
can justify a later baseline proposal. The current observed output is evidence
of failures and configuration change, not an accepted prepared-provider baseline.
This audit leaves App goldens, assertions, budgets and deadlines unchanged.
