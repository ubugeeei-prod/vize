# Configured roots remain authoritative over filesystem ignore files

Owning issue: [#3984](https://github.com/ubugeeei-prod/vize/issues/3984).

The configured CLI input collector used filesystem ignore rules in addition to
the tsconfig's `files`, `include` and `exclude`. A selected `.gitignore`, `.ignore`
or `.git/info/exclude` source disappeared from the authored program. An ignored
ambient declaration also disappeared, creating a false TS2304 in a healthy
importer while real TS2322 errors in the missing TS and Vue roots went unseen.

The original corpus preserves direct and inherited six-root projects, a Vue
script/template, an ambient global, excluded broken controls, all ignore files,
and clean/broken/repaired source bytes. Official TypeScript 6.0.3 and native
7.0.2 whole `--showConfig` and project-command packets are frozen. The authenticated
unpublished, superseded CLI H212692 / R37715578811 / source cut 6d26 reproduced
all default-run failures, while explicit selection checked the six roots and
reported all four exact authored TS2322 diagnostics. That older source artifact
is defect evidence only; fresh compiled-source and protected Actions remain
mandatory. Complete original inputs and stdout/stderr/status stay unchanged.

Configured scans now disable filesystem ignore rules. Unconfigured discovery
retains them. Existing hidden-root selection, supported extensions, tsconfig
exclusions and git-metadata protections remain in force. The walker also prunes
the existing generated-directory policy at directory entry and prunes implicit
package folders before traversing their contents. A literal package segment or
an include rooted inside that directory retains traversal; the complete existing
include matcher still decides the final roots. This conservative pruning avoids
walking ordinary wildcard-excluded installed dependency trees without narrowing
any literal selection. No new pipeline stage, serialized representation, cache
or cross-run state is introduced.

Literal package retention uses the existing glob platform case policy: ASCII
case-insensitive package names on Windows and exact case elsewhere. The same
boundary law covers all three package-directory names with uppercase includes;
a case mismatch must not prune a root that the configured matcher can select.

Four helper laws retain all three original selection corpora, ignored
ambient inputs, unconfigured ignore behavior and package-directory pruning. Two
public CLI laws must execute twelve Vize invocations and six official native
project commands, plus two native version and two whole config probes. Their
complete expected DTOs retain every file, diagnostic, program, compiler option
and count. Expected values are authored independently of current responses.
Optional complete runtime receipts reuse the existing configured-type capture.
The native source qualifier must retain all prior targets, environments, fields,
artifacts, timeout and budgets while adding this test target in its existing job.
Ordinary disable-TSGO returns provide no native-body acceptance. Full protected
runtime and unchanged instruction gates precede actual merge.

The separate generated-boundary corpus freezes normal-root, explicit target-root
and explicit codegen-root contexts, each containing ordinary TS and declaration
controls. The unchanged actual-main `3e493feb7f` collector and the unpublished
successor compile in the same bounded helper envelope and produce identical
whole vectors: a normal root excludes target and codegen entries; an explicit
target root keeps ordinary TS and plain declarations; codegen declarations stay
excluded in every context. Complete local helper packets, source-module hashes
and original vectors are archived. Path/extension adapters are test boundaries;
these local checks supply no complete CLI, native or protected qualification.
The pre-existing generated/codegen selection policy's general stock-TypeScript
authority remains outside this repair and unfinished under #3984.

The original literal `node_modules/selected/index.ts` case also exposes a separate
explicit-run workspace-package ownership failure in that older source artifact.
Its original inputs and complete failed packets remain preserved. This change
asserts its collector selection only; whole public graph qualification and a
separate current-source reproduction/repair remain TODO under #3984. That case
does not receive public CLI acceptance credit from the two six-root laws.

Complete project/build/watch semantics, declaration/map and packed-consumer
qualification, LSP/editor parity, installed replay and matched performance
targets remain unfinished under #3984 and #3957. This slice does not close those
issues or replace a legacy provider. Upstream projects remain read-only.
