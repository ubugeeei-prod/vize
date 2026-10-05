# Component-registration tag-name ranges

Paired issue: [#7979](https://github.com/ubugeeei-prod/vize/issues/7979), with
[the implementation decision](https://github.com/ubugeeei-prod/vize/issues/7979#issuecomment-5995407588).
The inspected baseline is actual main
`d8cd6a208b9aea02150b140a4b81b87222128a51`.

## Correction

The existing collector starts its component diagnostic at
`element.loc.span.start`, the opening `<`, then adds the tag-name length.
For the complete original `my-panel.vue`, this selects `<MyButto` at
columns 5–13. Advance that start by the single opening angle byte before
using the existing tag length: the diagnostic selects `MyButton` at
columns 6–14. No new traversal, parse, pipeline stage or serialization is
introduced. All component detection and registration predicates remain
identical, including imports, self references, Art variants and framework
globals. Message, help, severity, labels, fixes and unrelated findings keep
their existing contracts.

## Original inputs and controls

`crates/vize_patina/tests/fixtures/issue-7979/` retains the entire issue
body, both original SFC/config fences and their byte lengths and hashes.
The original file is copied unchanged into a real `.vue` input by the
public CLI test. Authored CRLF, Unicode-prefix, nested/self-closing and
registered/builtin/framework controls retain independent expected tag-name
spans. Lint JSON preserves its existing one-based Unicode scalar columns;
this correction does not change the editor's UTF-16 coordinate contract.

Five Rust laws compare every result/diagnostic field and the exact source
slice; the original is pinned to byte span `24..32`. The existing complete
Art registration laws retain all original input bytes, assertions and
registration behavior. Only their common affected start/end expectation
moves one byte, so these laws now enforce the same physical name boundary.
No historical input or unrelated expected result is regenerated.

The new normal tooling test requires an exact source-build receipt, keeps
complete inputs and raw stdout/stderr, compares all four public JSON
results, and checks the original ANSI location and name-width underline.
Its qualification count advances only after each whole result passes.
`nativeHandled` remains zero: this legacy range repair grants no native
replacement or fix-history completion credit.

First source head `e91af795` compiled successfully in Check37317455277,
but tooling job111787976540 rejected the new ANSI assertion: the genuine
raw report already had `my-panel.vue:3:6` and its eight-character name
underline, with VT color bytes between the filename and coordinates.
The original complete JSON passed; the remaining three CLI rows were not
executed before that failure. Preserve the entire raw report and compare
its visible text with the standard Node VT-strip utility. Production,
inputs, expected coordinates and all diagnostic contracts stay unchanged;
fresh exact-source Actions are required for the corrected harness.

## Delivery and remaining work

Source review and authored expectations do not establish execution.
Fresh exact-head automatic Actions must run the Rust laws and public CLI
test. The first-v0.433 publication hold has been lifted; this independent
PR remains Draft and off queue while its fixture-custody successor qualifies.
After that qualification, unchanged protected 104 instruction
ceilings and full suites, the actual signed reporter-credited merge, and
released public payloads remain separate requirements. Broader global
component configuration in #7978, native linter/history acceptance in
#6881, and the separate 10x target remain unfinished. The verified primary
reporter is `ubugeeei` (GitHub ID `71201308`); the meaningful source commit
includes the literal verified noreply Co-Author trailer.

## Fresh source and checkout custody

The replayed source `83df06a6b9bcc1d067f274f315d89d5284eec26f` and its
genuine prospective tree `926fd61f1873382c106f893e56032d93512e38a0` pass
Check37338548465. Four small JUnit artifacts contain 16,177 passes with
zero failures or skips, including all five range laws and ten complete Art
laws once. The four whole CLI JSON results and visible original ANSI range
pass using the exact source-built binary; `nativeHandled` stays zero.

Git previously left this new fixture directory's text conversion unspecified.
The scoped `-text` attribute now retains all seven authored files exactly
under `core.autocrlf=true`, including original LF and explicit CRLF bytes.
Production, every original/control fixture, full expected diagnostic fields,
all laws, CLI assertions and budgets remain byte-identical to source83df.
This meaningful checkout-custody correction requires fresh automatic source
Actions; the previous source's green execution does not transfer to its
new head. Actual protected 104/full-suite and signed-merge proof remain
pending. The [paired issue decision](https://github.com/ubugeeei-prod/vize/issues/7979#issuecomment-5998783276) records the same bounded correction.
