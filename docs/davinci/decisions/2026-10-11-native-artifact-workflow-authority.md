# Native artifact helper workflow authority

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The packing step added by #8570 must run the transport recipe from the executing
workflow, while `SOURCE_SHA` and `DRIVER_SHA` continue to identify the selected
CLI source. A PR checkout can precede that workflow and contain no packing helper.

The original #8582 native run 38114814234 selected source
`19a49d5823b348c63bffa9c5cb9b5e07e89cd659`, while its actual workflow authority was
`b5a9ee296ff9222421eba3e1018ab1de0dc543b6`. All seven original native/CLI/editor/project
qualification steps passed; packing alone failed on the absent checkout helper,
and the missing fallback output prevented the raw artifact upload. The complete
3,643,185-byte job log has SHA-256
`a0e87c80f4843b7d40752adef7dadad424ebcadee3fb70f8d46c70a5e878832a`.
The same transport failure occurred on #8584 run 38115118632 and #8579 run
38114944560; their failed source executions remain historical failures.

Inside the existing packing step, emit the exact original 22 raw paths before
any authority lookup. Require a complete immutable workflow commit SHA, fetch
that exact object if absent, and materialize the entry module, its relative
module import and its relative JSON path inventory into a fresh physical helper
directory. Every member comes only from that workflow Git object. Missing or
invalid authority fails the step; no selected-checkout or mutable-ref fallback
is allowed. The existing always-upload step retains the whole original raw
selection on failure, including hidden files, with its original compression.

The three transport modules, archive algorithm, strict reconstruction laws,
all original qualification steps, source identities, triggers, concurrency,
timeouts and runner remain unchanged. The already-authenticated #8570 Linux
payload contains 685,199 original files and 6,675,159,711 raw bytes; this change
claims no new packing speed result or original product failure repair.

Real Git fixtures execute the exact workflow shell with a helper-free old
checkout and poisoned checkout modules, then reconstruct every original packet,
alias and hidden byte. Malformed SHA, missing Git object, non-commit object and
each missing authority closure member refuse while preserving all 22 fallback
paths. Original damaged/missing/raw-symlink and shared-inode laws remain intact.
Local eight laws pass without skips; fresh exact-source Actions and protected
merge qualification are required before this workflow fix is complete.

The first adf537 source run retained a genuine source-length failure: the inline
workflow grew from 334 to 371 lines across the unchanged 350-line ceiling. Store
the same 22 literal fallback paths in a Bash array and print identical output;
the workflow now stays within that original limit. Retain the failed execution,
rerun the original length guard and all eight laws, and require fresh source
Actions without any budget or workload change.
