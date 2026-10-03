# L4 ordinary empty-component rewrite

Issues: [#6840](https://github.com/ubugeeei-prod/vize/issues/6840),
[#6838](https://github.com/ubugeeei-prod/vize/issues/6838),
[#6880](https://github.com/ubugeeei-prod/vize/issues/6880).

The preceding [L2 ordinary provider](./2026-10-04-l2-ordinary-empty-component.md)
supplies genuine original ordinary ScriptView/source/profile/Program/File and
sole direct-empty-default authority. The next bounded L4 provider accepts only
that sealed `VueOrdinaryEmpty`; neither a bare source block, caller-selected
coordinates, supplied component name nor `VueSetup` can substitute for it.

`module::ordinary::emit_ordinary_empty` shares the existing generated
`COMPONENT_BINDING` (`_sfc_main`). It checks the certified authored statement,
export keyword, default keyword and object ranges against the actual script
block, with ordered UTF-8 boundaries, before creating a Writer. Its only typed
failure is `OrdinaryEmitErrorKind::OriginalRange`, retaining the original span.
The provider introduces no parse, AST/source walk, serialization, new level or
trusted caller binding/span parameter.

Copy original script start through statement start as one linked leading gap
when nonempty. Replace statement start through object start with generated
`const _sfc_main = `, without an authored link. Copy the complete original object
through script end as one linked tail. Each retained original byte is copied
once; the empty leading gap creates no phantom link. Original prologue
directives, empty statements, leading/object/tail comments, CRLF and UTF-8 bytes
remain exact. The selected pinned Vue `genDefaultAs` prefix policy removes the
export/default prefix including intervening comments and whitespace; those
omitted bytes receive no source links. The original parser observation still
retains every comment and original range without mutation.

The returned `Writer<L>` joins the existing `ScriptPart::Body` path. ModuleParts,
setup emission, helper vocabulary, prepared render placement/attachment and
final default assembly are unchanged. The script provider creates no setup
wrapper or setup metadata. The existing assembler remains responsible for
statement delimiters after an authored trailing line comment; this provider
does not synthesize script-tail bytes or parse the prepared render fragment.
Recorded and NoLinks emission must produce identical text and helper sets.

After replay onto the actual refreshed L2 parent, read-only integration review
finds the inherited mandatory `ModuleParts::scope_id` absent from six owned
assembly literals. Each now explicitly supplies `None`, retaining the complete
module, document, link and original-owner oracles. This source correction adds
no scope authority or production API change; fresh hosted compilation remains
required before publication or acceptance.

Six genuine original-descriptor/parser/complete-File laws cover direct JS/TS
fragments, exact borrowed leading/object/tail bytes and link boundaries, omitted
prefix comments versus retained original observations, empty leading gaps,
Unicode/CRLF and original owner moves, whole emitted document and full-file link identity,
foreign/unsupported input refusal, semicolon-free script-only whole assembly and attachment of
a clearly prepared render fragment. Every synthesized binding, delimiter,
prepared render, attachment and final default byte remains unlinked. One
compile-fail example prevents a setup capability from entering this provider.
Prepared render tests supply assembly evidence only, not native target/product
or captured runtime acceptance.

Source formatting, diff and unchanged 350-line checks are performed without a
local Rust build/test or installation. Fresh exact-head configured Actions must
compile and execute these laws and doctests. Full protected suites, all 100
unchanged instruction ceilings and actual native Stack merge remain required.
No workflow, action, reference/native fixture output or acceptance budget changes
are included.

TODOs remain genuine product dispatch through this sealed provider and original
selected native File, complete module/map and actual Vue runtime capture,
broader Options API/binding/dialect support and compiler fix-history/default
migration. Product dispatch must independently reject an actual setup sibling;
this script proof does not establish whole-descriptor completion. #6880 remains
open and neither a legacy-backed shortcut nor default compiler migration is
introduced.

The corrected `1d94e21275e2b888f2db3e10559d63b459857073` source passes
Check 37135811757. After the preceding #7613 actually merges as signed
`90c1546c29b34fbc0e0d4ff3741e934354812d38`, its queued child #7618 retains
a stale feature-branch base and an UNMERGEABLE entry without a candidate.
The normal dequeue operation rejects that feature branch's missing queue.
Reversibly closing only #7618 releases its entry; live GitHub reports no queue
entry. Unstacking removes only the two remaining owned children from their old
record, leaving the merged historical prefix intact. Reopen #7618, retarget
fresh main and replay only its genuine L4 changes; preserve incoming original
handler providers and scope metadata. Re-register the remaining dependent
children as a new native Stack and require fresh exact-source Actions before
protected admission. This repairs metadata without acceptance credit from the
old source checks, queue reservation or prepared-render assembly tests.
