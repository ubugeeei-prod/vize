# Existing instruction-window diagnostics

Tracked in [#6868](https://github.com/ubugeeei-prod/vize/issues/6868), with
the CI roadmap in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The native canonical provider's second strict run still exceeded the medium
and stress-deep DOM compile ceilings. The earlier single-markup-machine
candidate (`ca028b7`) had four token-workload regressions; its final candidate
subsequently passed all one hundred ceilings and merged. Archive download
transport is unavailable in the current local runtime while decoded Actions
logs remain readable. The L2 performance repair still needs actual measured
function evidence.

After the existing three executions complete, the strict comparator reads
the already-produced first-run Callgrind files and reports only workloads
whose measured instruction counts exceed their unchanged ceilings. It
prints the top twenty object/function groups by exclusive cost and by
inclusive cost. Regular rows contribute self cost; the cost row following
`calls=` contributes the inclusive outgoing call-edge cost. These rules
follow the [Callgrind format specification](https://valgrind.org/docs/manual/cl-format.html).
Inclusive groups overlap and cannot be summed to obtain the window total;
recursion is reported as the recorded call-edge cost, without inventing a
cycle-collapse analysis. Inlined source-file changes keep their function
owner. Identical names within one object are grouped together.

The diagnostic accepts the existing single-part, Ir-only named dumps,
handles compressed function/object names and position columns, checks the
sum of exclusive costs against the measured total, and checks each selected
dump's workload identity and instruction count against the measurement.
Missing or malformed diagnostic input prints an unavailable message. The
original strict comparator then runs unchanged and still fails: diagnostics
cannot approve a regression or suppress an identity/methodology failure.

No workload is re-executed. The Cargo profile, Valgrind flags, guest context,
fixtures, registry identities, three-run equality protocol, measurement
JSON, benchmark windows and all one hundred numeric ceilings stay unchanged.
There is no extra compiler pipeline stage or artifact serialization.

Five laws cover independent nested/recursive cost goldens, compressed names
and object ownership, malformed input, failure-only selection, and the real
strict CLI. The CLI probe proves that successful diagnostics and unavailable
diagnostics both preserve the original hard failure, while within-budget
input neither reads dumps nor emits diagnostics. All seventeen diagnostic
and existing comparator laws pass locally. These are parser/runner proofs;
actual x86-64 function hotspots and any performance fix still require Actions.
