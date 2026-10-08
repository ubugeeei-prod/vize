# Qualify original literal package roots and cache transitions

Issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).

The original `tsconfig-ignore-3984/literal-dependency` corpus selects seven
authored TS, Vue and declaration roots, including the manifestless
`node_modules/selected/index.ts`. Its historical default-then-explicit run
retains a whole explicit failure: `Workspace package alias requires owned
directories`. That older default omitted the selected dependency. It may have
left an external cache leaf that the explicit owned-file write then correctly
refused. Current configured collection includes the dependency from its first
run, so fresh replay may differ. This is an inference, not a current diagnosis
or proof that the historical failure is repaired.

Create a tests-only genuine native Stack child of finalized configured-ignore
head `ba40e0c0efd28ddb78864dc4eef324efa41f834e`. Preserve every original input,
stock packet, historical failure, production guard, full recipe and budget.
Reuse the existing required `check_tsconfig_ignore_cli` target; no workflow,
target, stage or producer changes are needed.

The original default-then-explicit case now compares the complete independently
authored seven-root DTO for clean, broken and repaired phases. Its count comes
from the original golden root vector, preserving the existing six-root DTOs
unchanged. Separate fresh-explicit projects run all three phases. A separate
POSIX raw-link boundary repeatedly checks the existing direct six-root config,
then the original seven-root config, through clean, broken and repaired states
in one cache namespace. It adds only an unrelated raw endpoint and symlink;
the selected dependency remains manifestless and gains no invented package or
route authority.

All stock and Vize project commands conserve the complete raw package tree:
directory/file kinds, file bytes, readonly state and raw OS link/path values.
The separate POSIX control also conserves its complete endpoint tree. Persist
whole command packets and before/after trees to the existing optional capture
directory before asserting status, diagnostics or conservation, including a
missing-path receipt if a command deletes the tree. Expected diagnostics and
roots never come from these runtime receipts. The original six-root laws retain
their complete expected reports and gain the same additive conservation check.

On the required Linux native runner, five CLI laws execute 27 Vize commands,
18 official native project checks, five separate version probes and twelve
whole showConfig probes. This includes the original three cases and both new
transition controls. Ordinary disabled-native returns grant no execution
credit. Fresh exact-head Actions must retain whole failed packets before a
producer change is justified; ready parent prefixes can enter the queue
without this child's readiness.

If a current failure appears, preserve it and repair only independently proved
owned-shadow topology or completed-plan custody. Do not weaken the raw-directory
guard, treat a matching external endpoint as ownership, or manufacture a
manifest for the selected input. Existing raw-parent, retargeted-link,
retired-shadow, editor-union and overlap negative controls remain mandatory.

Current literal-graph qualification, protected delivery, installed acceptance,
complete project/build/watch and performance criteria remain unfinished. This
change does not close #3984 or #3957. Upstream projects remain read-only.
