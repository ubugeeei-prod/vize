# P3-6 — Flat select models (2026-09-26)

Model-bound `select` elements with direct `option` children now generate from
the checked S3 artifact. Option children are text-only; isolated options,
optgroups, conditional/loop option carriers, component children, and raw HTML
content select explicit legacy reasons. The guard follows the checked tree,
including its ownership, rather than guessing browser child addresses.

Models preserve the existing read/write target contract, including computed
members and `.number`. The shared emitter now uses Vue's `setValue` for bound
option values, preserving the raw object value that select models read.
Previously it wrote only a DOM string through `setProp` and lost object values.
Both lowering lanes share this correction.

Four TS-33 scenarios compare exact DOM, selected option flags, model state,
node identity and unmount with the pinned official compiler and runtime:
single selection with an unmatched external value, multiple array selection,
numeric assignments, and bound object values across user/external updates.
The runner can select options by index, so equal serialized object values
cannot hide an incorrect raw assignment.

Static option code and templates match the retained lane exactly. Dynamic
option children reserve different node numbers between lanes; their native
code is pinned separately and their runtime behavior is compared. Payload
mutation proves the checked model owns its assignment. Source-map snapshots
pin templates, option properties and option text, with unchanged code when
mapping is disabled. They do not expand model callback mapping precision.

Both prefix settings have zero legacy walks and expression reparses for the
four select floor cases. Existing allocation ceilings remain unchanged.
Parser agreement still checks the full fixture corpus, punctuation mutations,
and malformed select nesting. P3-6 remains open for the full surface matrix,
remaining constructs and performance exit.

Contract: [P3-6](p3-6.md).
