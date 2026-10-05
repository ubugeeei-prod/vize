# Report the formatter's document support accurately

Issue: [#7867](https://github.com/ubugeeei-prod/vize/issues/7867).

The original YAML and Markdown reproductions contain syntax/layout that the
current CLI only normalises partially. Reporting these documents as already
formatted incorrectly suggests equivalent structure-aware formatting.

Use the issue's explicitly accepted alternative: exclude YAML and Markdown
from default patterns and state that formatting them is not implemented.
Explicit selection returns the complete unsupported-format error and exit 1
before reading or writing those files. Other selected source files retain the
existing command behavior. The CLI help and public guide describe the actual
supported defaults. Private historical normalisers and their original unit
laws remain unchanged; their existence does not grant full format support.

The shared six-scenario legacy corpus preserves the original YAML and
Markdown bytes, alternate `.yml`/`.markdown`, a Markdown hard-break control and
an implemented TypeScript input. Source-built CLI check/three-write/recheck
observations compare every project byte, complete error vector, selected
count, exit status and original process stream. Default discovery selects only
the TypeScript file. Explicit YAML/Markdown and mixed inputs must fail and
preserve every unsupported document. Existing differential artifact retention
keeps failed observations as well as successes.

The root formatter command remains at its original 460-line ceiling; helper
extraction avoids ratcheting that existing file upward. Source qualification,
protected full suites, unchanged instruction budgets, actual merge and release
remain required. Full YAML/Markdown formatting remains unfinished and is not
credited by this correction.

Initial exact-source Check [37259167045](https://github.com/ubugeeei-prod/vize/actions/runs/37259167045)
retains its failures: three old CLI tests expected the former partial YAML/Markdown
normalisation, the help snapshot retained unsupported default patterns, and the
public guide grew above its existing 375-line ceiling. The successor keeps the
original three authored inputs and asserts complete explicit refusal, empty stdout
and unchanged bytes across check/three writes/recheck, adds alternate extensions
and Markdown hard-break coverage, updates the help snapshot to the observed full
output, and puts the support description into the guide's existing paragraph.
Private normalizer laws, six-scenario corpus, source caps and instruction ceilings
remain unchanged; the original failed run is not a successful qualification.

The actual-main replay on `8f667ea070bb90d57ac7125db35d791025f746e2` preserves the complete incoming protected prefix and every owned production/runtime/corpus/witness byte from `0eca9f09315d517d007584a3c433de88a1354fdb`. Original authors, reports and ceilings remain intact. Earlier qualified heads remain historical receipts; this replay needs its own exact-head source and protected reports before actual merge/release. The public CLI guide retains the incoming Ready progress explanation verbatim and only the reviewed formatter-support paragraph differs from this main; both guides remain exactly 375 lines.

The genuine bf91 protected-tail composition conflicts only in the shared
canonical footer. The actual736 main replay retains every incoming decision
and all owned source/corpus bytes; this clause now occupies its relevant
distinct canonical location at350 lines. Fresh exact source and protected
delivery remain required; no earlier acceptance or publication is transferred.
Paired [placement decision](https://github.com/ubugeeei-prod/vize/issues/7867#issuecomment-5988293397).
