# Nuxt Oxlint configuration root (#7983)

## Problem and boundary

The complete original report reproduces `../**/dist` in the default generated
`.nuxt/oxlint.config.json`. Oxlint rejects parent segments in global ignores;
removing ignores leaves the parent-relative overrides unmatched. This is the
regression of #7251, not a compiler or native-linter semantic change.
The [paired Issue decision](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5991355930)
records the original corpus and mandatory execution gates.

The [Oxlint configuration contract](https://oxc.rs/docs/guide/usage/linter/config-file-reference)
roots ignores at the config directory and cannot match files outside it.
Pinned Oxlint1.78 `config_store.rs:118–136` uses a config-relative path inside
that directory and the actual absolute path outside it for overrides. A blanket
`**/` prefix would broaden overrides while leaving outside global ignores broken.

## Selected repair

Generate `.oxlint.vize.json` in the Nuxt project root by default. Keep the root
plan globs unchanged when both directories coincide, and prepend only the exact
relative plan-root namespace when the config is in an ancestor. Outside-layer
overrides/exclusions use their actual absolute paths, matching Oxlint's original
path selection. Outside global ignores and below/outside-root config locations
are rejected before writing an invalid artifact. This explicit migration
replaces silently ineffective settings; it does not widen the lint target set.

The reserved default file contains the recognized settings marker
`settings.vize.generatedBy: "@vizejs/nuxt"`. Exclusive initial creation cannot
replace an authored collision; later writes require the exact ownership marker
and a regular file. Malformed, unowned and symlink collisions remain unchanged.
Ownership follows the resolved reserved root path even when explicitly named
with a relative or absolute option; peer review identified and closed the
[option-presence bypass](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5991453040). Other explicit custom files retain the existing
regular-file/atomic write contract and are unmarked. Add the generated file
to the project's `.gitignore`.

Auto-init still preserves existing project/ancestor Oxlint and Vite configs.
Only the complete exact former generated default `oxlint.config.mts` loader
can migrate automatically to the new artifact; any authored edit or symlink
is preserved. A custom ancestor config with a new loader needs `autoInit: false`
so that a root loader cannot reinterpret the config's different glob base.
Existing explicit plugin specifier URL resolution remains unchanged.

## Original source and execution gates

The new linter differential corpus retains the whole original Issue body
SHA256 `b8f105a611cf70a62e9e10526592c5dcb53c7ac018e65ceb4a618cfdb5637769`,
renderer, failed JSON, two original paths and identical full SFC
SHA256 `aacf0733b25709e6b813e6af1909e701762d80c2aaa5e7c6f8f0dcf8f4548ff0`.
The `.vue.txt` is storage only: the mandatory automatic recipe copies these
exact bytes to physical `.vue` inputs and requires authentic source-NAPI calls.
Original upstream whole-artifact recordings and all unrelated assets stay
unchanged; current default generation adds only the explicit ownership marker
to the original root-relative plan, with the incorrect rebasing removed.

The existing Nuxt3 automatic job builds the source native addon, current Nuxt
integration, lint plan and Oxlint bridge. It retains the old compiler
client/SSR/browser controls and #7999 manifest/prefetch packet, then uses the
unchanged Nuxt3.19.3 and Nuxt4.5.2 dependency cohorts for finite lint probes.
The unchanged workspace Oxlint is1.78.0; reporter1.75/1.86 are not separately
executed or credited. Each cohort runs actual module default generation and
seven CLI cases: default, original startup failure, original unmatched
overrides, original root renderer, page exclusions, ancestor namespace,
and outside-layer/neighbor controls. The old invalid config remains a failure.

Full raw configs, CLI process status/stdout/stderr/reports, current whole package
dist files and source-NAPI load/input/options/return events are retained.
Complete native inline-style results pin the original message/help and authored
attribute span76–94, line6 columns7–25. Full CLI diagnostic identity/message
and exact visited-file/warning/error counts enforce ignores and ordered
overrides; nondeterministic timing fields are retained without speed claims.
Close Nuxt before sealing native events and remove only the probe's borrowed
dependency links/project after its finite child completes. Original projects
and lockfiles remain unchanged. Nuxt2's old root-placement workaround is
removed so its existing complete process-flag controls use the actual default.

Local configured formatting/lint, original corpus laws and emitter laws pass.
The emitter run includes the existing dummy-JS Oxlint test; it gives no source
NAPI credit. Local generation laws cannot load the absent workspace dist and
remain unexecuted. Fresh exact-source Actions,
authentic whole packets, unchanged protected100+4/full suites, actual signed
reporter-credited merge and supported publication are still required.
No native/compiler/default-history migration, browser/SSR expansion or
performance result is claimed by this lint-config repair.

## Frozen review and actual-main replay

[Independent read-only review](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5991542057)
clears private `bbcd2298df59f85652f11c5977bd912021806c4f` over actual `69b2c8c1`.
The receipt SHA256 is `c357b96a25d1336be9affebffe27989af54462a26f23647ebe58554accf01d34`.
It checks all25 source hashes, whole original Issue/corpus, reserved-path and
namespace/refusal controls, full physical CLI/source-NAPI recipe and cleanup.
The genuine replay onto actual `47dcd54e31e24b8660edacb1a5dde63099eb5a30`
preserves all21 non-doc blobs and complete incoming decisions. Source review
grants no execution credit; fresh hosted native/full protected delivery and
supported publication remain pending.

## Initial source integration failures

[Original failure custody and correction](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5991673501)
retains source4a/actual mergeea2b over9f. Check37288937359 rejects two unhandled
Node corpus registrations; explicit `void` preserves all assertions and inputs.
Nuxt37288936439 passes inherited source compiler/SSR/Chromium and whole SPA
controls but public ESM-only Nuxt exports reject the probe's CommonJS resolver
before the first lint case. A project-local literal ESM host imports both
public APIs through ordinary import conditions, retaining its complete bytes
without private paths or fallbacks. Authentic artifact11336140619 keeps raw
failed process, actual source/native custody and four exact original hashes.
Nuxt2 run37288936800 passes actual corrected-default generation, full plan/
Vite+ controls and webpack/SSR; its installed-native envelope is distinct.
Production/oracle/corpus/caps remain unchanged. New fourteen source-linter CLI
cases and genuine successor/protected qualification remain pending.

[Modern public Nuxt options](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5991806789)
record actuala029/merge7386: check-js and old Nuxt2 controls pass, while the
new Nuxt3 probe reaches public ESM but finds no root artifact before its first
CLI case. Direct Nuxt3/4 `loadNuxt` uses `cwd`/`overrides`, as pinned primary
3.19.3/4.5.2 source proves; copied Nuxt2 `configOverrides` leaves the original
fixture's lint:false in effect. Use real overrides/dev:false and assert the
actual root, with no fallback artifact or oracle adjustment. Recipe350 retains
all original/production/native-loader/cap bytes and authentic failed packet
11335759261; fresh fourteen CLI/native cases and protected delivery are pending.

[Original addon and pinned JSON contract](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5991932826)
retain actual7f/c1d failure: the existing addon hook requires `getConfigs()`
descriptors, not raw plan rows. One named descriptor returns a fresh complete
unchanged original three-row plan. Pinned Oxlint1.78's JSON reporter and actual
old Nuxt2 reports expose no top-level warning/error counters; strict equality
of the entire expected code/file/message/severity vector already enforces
all diagnostic cardinality and severities, while real number_of_files and
complete source-native counters/results stay exact. Remove only those absent
metadata-field assertions, without changing any expected case/input/value.
Observer349 and production/corpus/budget bytes stay preserved; failed artifact
11336556257 remains failed and new fourteen-case execution is still pending.
Independent read-only delta review clears runner blob
`e2a28f3697779a57ffecd8743afcaf08b907deb7` and diff SHA256
`b0cd9c114c621a1a91367632e93799eb2531ca0785d4126435950527003ebac4`;
it confirms complete vector equality excludes every extra/error/duplicate
diagnostic and grants no runtime credit.

## Original filter and full-source transport

[Paired source decision](https://github.com/ubugeeei-prod/vize/issues/7983#issuecomment-5993038459)
The current0fa source Check37292032191 fails the old module-addons test's
former artifact filename; changing only that path retains all plan assertions.
Nuxt37292031544/job111704430319 executes source3b24 over2cb and fails the
first CLI case with zero files. Authentic11337510803 retains its valid root
config, full original bytes and35 source-native events: one load,34 catalog
calls, zero lint calls. These are failed packets, not fourteen-case success.
The existing bridge copies every Vue beneath ignored node_modules and changes
its override identity; retaining #7130's full-source location bridge is required.

The same installed Oxlint selects original files with --debug files after its
CLI/VCS/config predicates and validation. Transport only those selected Vue
files outside both roots; exclusively create a sibling JSON config so original
plugin resolution, fields and ordered values stay rooted. Original rows exclude
owned copies; adjacent rows retain the supported original comparison domains.
A second real selector must equal the complete projected original set, including
untouched non-Vue paths. Every selected path joins the existing single candidate
enumeration. No JS predicate, no-ignore, expanded namespace or skipped bridge
substitutes for the original engine. Clean only owned files on every prepare
failure and execution completion.

Pinned1.78 command/{lint,ignore,mod}.rs establishes all scalar and boolean
arities. Preserve each whole value, flag order and -- separator; remove only
original targets and pass the selected physical absolute paths. Plugin flags
consume no value. Unknown/compact forms and alternate inspection/update modes
refuse before mutation. Inherited configs, type/import/tsconfig graphs,
suppression state, non-POSIX roots, CR/LF/backslash candidates and unsupported
outside wildcard domains fail actionably. Discovered/dynamic configs retain
their old route, with no new filtering guarantee. The newly authored outside
control keeps logical paths/source/expected reports but passes exact absolute
filenames because the pinned CLI rejects positional parent-directory components.

Only the new, unexecuted observer changes: existing plugin/format/path helper
sources establish no redundant mapped-message annotation and absolute reported
outside paths. Full code/message/severity/cardinality and full labels remain
strict: original start76 plus independently authored encoded-source prefix,
length18, line6/column7. Primary SHA256s are plugin.ts eca74b5c9f89147c8741df39aceb51c2365c8a142ad52dcc7738a955edd2b8c2,
format.ts 767b7f689ae84c1d4f29b6c57685a3c416d7e35b6156a21ef4878241c331bbe4,
and old helper837a1f35c31ce78c2e896526b5f4ef1d427d1151870d683d2772507b0cfbd130.
All old snapshots/original inputs/native whole-span results stay unchanged.
Each CLI packet records monotonic full-process elapsed time; extra startup cost
is unmeasured, with no native-stage/performance claim.

Private15-path source receipt03453c475af62075a85c55d0a2c0eb80196fcbb5e12cb30055a917a2e91f5691
binds the reviewed successor over0fa. Configured formatting/lint and15 pure
controls pass, including all scalar/bool pairs, Vue-shaped values, separators,
unknown forms, non-Vue backslash/slash twins, complete namespaces and cleanup.
Independent source-only review clears packetf093dbff19c2ce2207c7f28932ffa5be80b0a35d2c356ccfdb798bc3f2cb8604; all15 source hashes, original corpus and7130/plugin/format bodies are authenticated. Fresh original fourteen source-NAPI cases,
unchanged #7130/full integration suites, protected100+4/full Rust and actual
signed merge/publication remain required. No runtime credit transfers from0fa.
