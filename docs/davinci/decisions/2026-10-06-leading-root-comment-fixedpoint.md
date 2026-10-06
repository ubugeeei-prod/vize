# Leading root-comment fixed point

Issue [#8117](https://github.com/ubugeeei-prod/vize/issues/8117) records the
public Habitica task SFC whose two adjacent ESLint comments move before
`<script>` on the first formatting pass, then gain a blank before that block on
the second. [The paired decision](https://github.com/ubugeeei-prod/vize/issues/8117#issuecomment-6013386949)
uses the existing root-comment grammar for first-block adjacency too. An authored
blank **after** initial comments keeps them as a standalone document prologue.
The rule applies to every block and comment text/count; the parser, checked source
ranges, first comment close, block sorting, no-block guard, BOM and EOL paths are
unchanged. No extra parse or product pipeline stage is added.

The complete original is `HabitRPG/habitica` revision
`19b2ce531c3bb1c3f95a75388b1176b345cbbca2`,
`website/client/src/components/tasks/task.vue`: 34,878 bytes, SHA-256
`58d63662f7743632e532e57f7bf0eb0a61bcb7ca536d975b4a29feeb87a8186b`,
Git blob `61ea3f95f4dd478df10828ee7d967596784650c6`. Its entire original
497-byte GPL-3.0 license notice accompanies the carrier. The official
[Matrix run37439069427](https://github.com/ubugeeei-prod/vize/actions/runs/37439069427)
actually ran source `d309824a07cce6577ef8aad80b61d940b995f6e3` and recorded
1 idempotence violation among 1,560 files and zero waivers. That historical
observer retained first-difference excerpts; whole historical fmt1/fmt2 are absent.
The new source branch starts from signed main `b96599a160a9af8083236fb2b19aaf2d2366bc4b`.

This behavior repair changes exactly one already-qualified CURRENT role among
300 original formatter history cases: the sorted-comments SFC. Both original
input and historical captured output stay immutable. Its independently authored
current expectation now keeps the initial template note with the template after
sorting, just as the script note stays with script. Three #7877 regression
expectations change for the same grammar: sorted comments, adjacent first comment
(formerly named `standalone-prologue`), and unsorted blocks. All other20 expected
outputs and all23 original source carriers stay exact. The complete old cases
manifest and superseded current snapshot are separate immutable evidence, and the
changed rows retain their earlier current expectations. Current admission accepts
only the corrected whole output; an explicit negative rejects the superseded
#8108 current output too. The old standalone interpretation has been corrected;
the new authored-blank control retains the standalone law.

Twelve independently authored whole controls cover first template/style/custom
blocks, sorted and independently attached groups, standalone and text prologues,
multiline Unicode, BOM, CRLF and comment-only documents. Rust and existing tooling
Actions require three whole formatting passes, changed truth, read-only initial
and final checks and full expected bytes for controls. The original Habitica
carrier requires full fmt1/fmt2/fmt3 equality and exact next-line-comment adjacency;
its large formatted output is observed, not a regenerated golden. The observer
retains every whole source, process result/stdout/stderr and compiler diagnostic
and DOM/SSR module, including failures. Existing semantic comparison checks the
complete template and SFC block envelope; the supported small controls also retain
full independent DOM/SSR states before and after formatting. This is not a full
Habitica application/runtime qualification. Actual hosted/protected tests, signed
merge and installed-public release remain required; native formatter handling is
unsupported and no performance claim is made.
