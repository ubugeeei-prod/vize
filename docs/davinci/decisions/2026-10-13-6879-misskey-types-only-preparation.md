# Misskey SDK declarations in the shared typecheck fixture

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

## Decision

Run the pinned Misskey SDK's declaration-only emitter in the existing dependency
preparation hook, before either Vize or `vue-tsc` checks the shared fixture. Keep
the original frontend config, corpus, frozen lockfile and divergence budgets.
Refuse a failed emitter or a missing, empty or non-regular SDK declaration entry.
The existing preparation receipt retains the actual command, exit status and
output hashes. This change prepares the fixture; it changes no checker behavior.

## Reproduction and source evidence

The original report at Vize `2817a0aa13518dd46c16bc8bf56ac831f68c24dc`, for
Misskey `810faa8e5db4bbf9d6d408a61df148559340ee99`, records a successful frozen
`--ignore-scripts` install with no preparation command. Its 487 false negatives
include 278 unresolved `misskey-js` imports; it also records 272 false positives.
The whole report is 241157 bytes, SHA256
`98a8d694bad470933ca00a1035f29851707fca976468b71f8b33edca9a18908d`.

The pinned [SDK package](https://github.com/misskey-dev/misskey/blob/810faa8e5db4bbf9d6d408a61df148559340ee99/packages/misskey-js/package.json)
exports `built/index.d.ts`, which is absent from that Git tree. Its
[build script](https://github.com/misskey-dev/misskey/blob/810faa8e5db4bbf9d6d408a61df148559340ee99/packages/misskey-js/build.js)
already separates declaration emission from JavaScript output:

```sh
pnpm --filter misskey-js exec tsgo --project tsconfig.json --outDir built \
  --declaration true --emitDeclarationOnly true
```

The [SDK config](https://github.com/misskey-dev/misskey/blob/810faa8e5db4bbf9d6d408a61df148559340ee99/packages/misskey-js/tsconfig.json)
includes its source tree, and all five generated schema sources under
[`src/autogen`](https://github.com/misskey-dev/misskey/tree/810faa8e5db4bbf9d6d408a61df148559340ee99/packages/misskey-js/src/autogen)
are tracked. Their whole bodies match their pinned Git blobs. No backend build,
API-schema regeneration, install scripts or JavaScript build is needed for this
selected emitter. The frozen workspace pins its own
`@typescript/native-preview` at `7.0.0-dev.20260116.1`; its main package's source
archive matches the lockfile integrity and resolves the installed platform
package's native executable.

## Verification and unfinished work

The dependency-preparation regression runs the existing native Rust command and
checks failed, missing and empty emitter output plus a successful receipt. Its
controlled package-manager stand-in proves refusal behavior; it earns no actual
compiler or real-project acceptance credit.

Hosted qualification must capture the actual fixture-local provider and
executable, emitter command and status, generated declarations, and unchanged
source/config/lockfile. Compare the full Vize and `vue-tsc` diagnostic vectors
under the same fixture. Prove their actual declaration reads separately: Vize's
owned input manifest and a shared working directory do not establish transitive
program loads. The preparation receipt alone does not establish them either.

The original 487/272 divergence counts remain unresolved until fresh causal
evidence shows the measured changes. This decision supplies no typechecker
migration, API/LSP, performance, merge or release-completion credit. The full
Real Project Matrix and release acceptance remain required.

The prepared successor composes literal main
`bd5702c0a4d5fb3bb64487794fc45f6f876e6e38` without altering the emitter or
refusal controls. Removing only the Misskey baseline addition reconstructs the
entire current registry, including every original corpus/config/pin/budget.
The shared foundation retains every current-main byte and adds only the existing
112-byte decision clause at350lines. Old source run38342813975's five-minute
planner setup cancellation remains historical; the merged direct-Node planner
requires fresh source Actions on the actual composed head. Hosted native
preparation and full diagnostic/declaration-load qualification remain pending.

## Literal Star source counterpart

Source Check `38354748231` at `08eb7f0e` genuinely refused the registry's
439-byte declaration-preparation addition in both tooling owners. The original
Star322 gate correctly requires the entire old registry, so it remains unchanged.
The named `misskey-registry-source` gate first authenticates all 412 literal-BD
source counterparts, regular committed and physical bytes, and its own selected
helpers, metadata and entrypoints. Its only three successors are the registry and
two exact test entrypoints. Removing the one 18-line Misskey baseline insertion
reconstructs the entire literal-BD registry; every neighboring corpus, pin,
config, lockfile and budget remains exact.

Only after this actual-source gate passes may either entrypoint execute its whole
unchanged original literal-BD test body. The original Star322/290, JS93 and eight
custody laws and all original negative controls remain in that immutable frame.
The shared counterpart snapshot utility selects raw regular files and grants no
admission. Additive controls reject missing, mutated or executable physical
source, symlink ancestors, raw/physical hybrids, neighbor budgets, stale or forged
packets, replacement refs and ambient source overrides. This relation is source
custody only; it adds no emitter, diagnostic, native runtime, original400 or
performance acceptance. The actual emitter and complete declaration-load and
diagnostic comparisons remain unfinished as described above.

Source Check `38361840157` at `c8845090` retained the successful source laws but
failed the zero-warning budget because the counterpart selector and source test
used default `.sort()`. Their explicit comparators use only UTF-16 relational
ordering (`a < b`, `a > b`), preserving every selected name and table comparison.
Removing the three comparator arguments reconstructs both entire prior files.
The utility successor is 9654 bytes, SHA256
`850682aade0e9dc3e37ca7629ca0ba18e966aa9bb6cb2d290a96d8e6ed67a3c8`;
its only change is that comparator. All 412 original source pairs and the 427-body
custody scope remain unchanged; the loaded-helper checks authenticate the whole
new committed utility and test bytes before any original-law replay. No warning
budget, source refusal, corpus, registry, emitter or acceptance assertion changes.
