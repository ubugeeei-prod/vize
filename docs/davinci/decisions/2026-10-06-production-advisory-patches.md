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
unchanged. Stable production catalog entries move together to 3.5.42; literal
beta/RC and independent test aliases remain. Existing Forge/Braces patches,
proofs and full-report audit requirements remain unchanged. Existing fixtures,
expected outputs, performance caps and workflow gates remain mandatory.

## Qualification and delivery

Frozen lock-only validation passed on pnpm 12.1.0; JavaScript syntax and source
custody are bounded checks, not runtime acceptance. Fresh exact-head Actions
must prove the installed graph, genuine compatibility controls, unchanged
production/cargo audits and all protected Rust/corpus/instruction suites.
Actual signed merge and next release remain unfinished until externally proved.
