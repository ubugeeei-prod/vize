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
