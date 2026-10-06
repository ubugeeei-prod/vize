# Production advisory patches, 2026-10-06

Tracking: [#8079](https://github.com/ubugeeei-prod/vize/issues/8079).

The required production audit began failing after 11 severe advisories were
published on 2026-10-05. Check 37398185879 job 112059063033 retained the full
13-entry report: the 11 new findings and the two existing independently
source-remediated Forge/Braces findings. Other current heads fail the same gate.
Earlier green executions do not qualify the newly disclosed dependencies.

Use official patched packages rather than relaxing the audit policy:

| Actual affected dependency            | Selected patched version | Official advisory                                                                                                                                                               |
| ------------------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| simple-git                            | 4.0.1                    | [x6jw](https://github.com/advisories/GHSA-x6jw-m9v5-85vh), [g4wm](https://github.com/advisories/GHSA-g4wm-2vf7-vfgr), [858h](https://github.com/advisories/GHSA-858h-whjf-mvg5) |
| @simple-git/argv-parser               | 2.0.1                    | [v5rq](https://github.com/advisories/GHSA-v5rq-49vh-5v5c)                                                                                                                       |
| seroval                               | 1.6.3                    | [jp82](https://github.com/advisories/GHSA-jp82-f5mq-hwhp)                                                                                                                       |
| source-map-js                         | 1.2.2                    | [68fv](https://github.com/advisories/GHSA-68fv-2mgg-jv7q)                                                                                                                       |
| proxy-addr                            | 2.0.8                    | [jqcg](https://github.com/advisories/GHSA-jqcg-44mw-7w3h)                                                                                                                       |
| postcss-selector-parser 7             | 7.1.6                    | [rj75](https://github.com/advisories/GHSA-rj75-hqrm-r3gf)                                                                                                                       |
| tinypool 2                            | 2.1.2                    | [5gmw](https://github.com/advisories/GHSA-5gmw-xhrv-c9v3), [85c8](https://github.com/advisories/GHSA-85c8-ppgw-ccpr)                                                            |
| Stable production Vue/server-renderer | 3.5.42                   | [g2v6](https://github.com/advisories/GHSA-g2v6-rqmx-r4w6)                                                                                                                       |

Simple-git 4.0.1 itself pins vulnerable argv-parser 2.0.0, so its actual child
is explicitly overridden to 2.0.1. Its required args-pathspec child is 1.0.4.
The major version removes the default ESM export. Authenticated official
Nuxt Devtools 3.4.1 still imports that default, so the package patch changes
only its import to `import { simpleGit as Git } from 'simple-git';`.
No unsafe argument/configuration opt-in is introduced.

The existing Node 22/24 engine job additionally executes the genuine initialized
Devtools `generateAnalyzeBuildName` RPC on a temporary Git repository. Complete
clean, tracked-dirty, restored, untracked-dirty and restored outputs must match
`branch#shortHEAD` with the exact `-dirty` suffix. Initialization guards cannot
skip the consumer and timestamp fallback fails. This proves handler/API
compatibility; it does not claim websocket transport coverage. Full stable Vue
SSR output retains safe attributes and rejects carriage-return attribute names.

Lock-only resolution generated the patch graph. Its unrelated changes to old
native publication pins, Nuxt ESLint RC peers and literal Vue 3.5.38 peer
selection were restored. Every retained same-version package metadata entry is
unchanged. Four production Vue dependency roots use a separate3.5.42 catalog and Nuxt's
production3.5.41 Vue resolves to3.5.42. Keep the full original3.5.35 dev/oracle
provider graph, Test Utils RC9/optionalSSR41 and all literal beta/RC aliases.
Every production snapshot path must exclude the vulnerable renderer; old
independently pinned dev oracles supply no production-audit exception. Existing
Forge/Braces patch bytes, crypto and full-report audit requirements remain. Existing fixtures,
expected outputs, performance caps and workflow gates remain mandatory.

## Qualification and delivery

Frozen lock-only validation passed on pnpm 12.1.0; JavaScript syntax and source
custody are bounded checks, not runtime acceptance. Fresh exact-head Actions
must prove the installed graph, genuine compatibility controls, unchanged
production/cargo audits and all protected Rust/corpus/instruction suites.
Actual signed merge and next release remain unfinished until externally proved.

The initial qualification helper incorrectly used require conditions for the
import-only Nuxt/Devtools roots. Peer review caught this before acceptance;
the successor uses existing Nuxt Kit's exsolve with explicit node/import
conditions and actual importer paths. No dependency or oracle relaxation is
introduced; fresh corrected-head Actions are required.

A second peer check found two remaining Test Utils compiler peer changes.
Both retain the entire original compiler-dom RC9/optional SSR41 dev-oracle
graph. Playground Vue RC6 and literal3.5.38 remain unchanged. No production
consumer reaches the old renderer; no peer removal masks an audit finding.

Initial Check37400254814 failed installation with ERR_PNPM_INVALID_PATCH;
the unified diff lacked the Git header required by pnpm. The successor adds
only that header/blob identity and updates exact patch hashes. The authenticated
consumer change remains one import; initial audit/consumer execution did not run.

Corrected-head Node22/24 installed and initialized genuine Devtools but then
failed Nuxt4 server initialization: the framework's intentionally direct Kit3
loader was selected. Preserve that product dependency; resolve and verify
Nuxt4.5.1's own Kit4.5.1 through its import path for qualification. No schema
workaround, mocked handler or whole expected output change is introduced.

The resolver bootstrap uses Nuxt's public package.json export and its own
declared exsolve dependency, preserving the framework Kit3 bridge without
borrowing an undeclared hoisted resolver. Actual runtime and Kit4 manifests
are checked before the genuine load/whole-control execution.

Actual07cf production audit removed all11 new advisories (moderate0/critical0,
high2 retained), then its installed Forge attestation rejected pnpm12's scalar
patch hash. The reader now recognizes only exact old hash/path or exact new
scalar hash, both bound to the reviewed workspace path and unchanged patch/RSA
bytes, registry/source identity, consumer graph and crypto laws. Every old
rejection remains; new wrong scalar/hash/path/extra-field controls fail closed.
No audit exception or requirement changes; fresh successor acceptance is required.

Actual7a Node22/24 passed all five genuine Git outcomes and both complete Vue42
SSR controls. Rust/native/SSR failures stopped at immutable Vue35 identity
guards; restore their complete original environment instead of changing those
references. Current whole UI42 execution also exposed two real NativeSelect
hydration failures: multiple selection lost its selected options. Vue39's
hydration dynamic-prop fix patches the authored `value: undefined` after option
hydration. Omit that value key in multiple mode; retain single-value binding,
option-selected ownership, event/form contracts and every original assertion.

The existing UI test script runs the full unchanged suite first under the
original35 configuration, then under a separate42 configuration anchored to the
real installed production example. All nine compiler/runtime/SSR package paths,
versions and manifest/entry hashes are attested; plugin-vue executes that actual
compiler, and the complete NativeSelect main/template transformed modules are
retained in the Actions stream. Whole original synchronous SSR/hydration,
node-identity, diagnostics, selected-state and interaction assertions remain.
This is no browser-transport or arbitrary mode-transition claim. Fresh installed
whole-suite/production audit/crypto/native/full protected delivery is required.

Two independently authored single-select SSR controls retain complete immediate
value/options/form states for an empty placeholder and a nonfirst value, plus
all form/select/option node identities and the full empty warning/error vector.
They run in both35 and42 without changing any original test or adding a tick.
Paired decision: [6007924307](https://github.com/ubugeeei-prod/vize/issues/8079#issuecomment-6007924307).

Actual0fad passed the installed production npm/cargo audit and Node22/24 genuine
consumer controls; full JS packages stopped only in the newly authored done
control before hydration (3807 pass/1 fail, including all original3806). The
pinned HappyDOM20.11.2 parser selects option index(selected-count minus one)
instead of the last selected option's actual index. Its auto-selected blank
plus authored selected third done option therefore produces premount todo.
Characterize that provider defect separately from standard HTML/product
semantics; verify the original selected-attribute vector and retain complete
SSR HTML, parsed option attributes/state, provider/version and actual loaded
component render functions. The immediate posthydrate whole done/empty states,
forms, node identities and diagnostics remain unchanged and require actual
fresh execution. The failed42 second suite never ran; no42 credit transfers.

Current0fad native computed-inlay controls stopped before fixture/oracle/RPC:
Nuxt's implicit physical Vue41 provider had moved to production42. Declare the
historical reporter authority explicitly as DEV-only `vue-computed-inlay-oracle`
(`npm:vue@3.5.41`); the actual pinned resolver selects realVue41/TypeScript6.0.3
without changing any original provider guard, README, input or whole native
oracle. Fresh installed physical package.version41 and all original semantic,
alias/UTF16/protocol vectors remain mandatory. Every old renderer stays outside
complete production reachability; unchanged whole audit policy still applies.
