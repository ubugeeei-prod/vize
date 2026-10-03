# Native default Vue 3 lint tag-kind receipts

Issues: #6835, #6836 and #6848.

The original L1 Element has no previously retained component kind. The current
registered SFC lint facade classifies its authored source using default Vue 3
lint grammar, rather than DOM table membership or runtime component resolution.
Importing its legacy semantic helper, substituting `is_html_tag`, or exposing a
caller-supplied category would not establish a genuine lower provider.

The selected native parser now records that lint kind during its existing
complete-header resolution. The original header event carries the private fact
into its original opening Token. Only the genuine Descriptor-selected owner
invokes the const-enabled construction; normal/compatibility and other dialect
constructors keep const-false construction and absent facts. There is no second
Descriptor observation, source reparse, AST walk, pipeline stage or serialized
handoff. Source blocks, parser observations and original grammar ownership stay
with the same privately paired Component.

`NativeElement::lint_tag` obtains a short readonly receipt over the original
Component and exact original Element. It exposes the recorded lint kind and the
absolute original tag-name span, without allowing raw element/source/kind/profile
insertion. It grants neither clean markup, Descriptor selection, File completion
nor compiler/runtime tag resolution. Arbitrary genuine non-selected Component
construction returns a typed unavailable-kind refusal. Missing opening framing
and unmatched source provenance refuse; original observations are retained.

The recorded grammar follows this order:

| Authored tag/header                                                                             | Default lint kind             |
| ----------------------------------------------------------------------------------------------- | ----------------------------- |
| Exact `slot`                                                                                    | Slot, including frozen scopes |
| Exact `template` with an originally resolved `if`, `else-if`, `else`, `for` or `slot` directive | Template                      |
| The six capitalized Vue built-ins or a first Unicode uppercase character                        | Component                     |
| Every other authored tag                                                                        | Element                       |

Slot shorthand and complete directive arguments/modifiers use the existing
native typed head decomposition in that same header visit. Case is preserved;
nonstructural lookalikes do not become Template. Lowercase custom/unknown tags,
`component`, deprecated `marquee`/`blink` and lowercase built-ins are Element in
this lint grammar. DOM namespace and opaque `is`/`:is`/`v-is` values do not change
it. This is explicitly not the compiler's dynamic/custom component admission.
The lexer admits an ASCII letter as the initial tag byte; tests do not fabricate
Unicode-first NativeElements that the genuine parser never constructed.

The native lexical policy accepts modified/argument pre heads, while the default
registered L2 facade treats every parsed Pre head as freezing for bindings. Its
original tag classification separately inspects raw structural template heads,
even on the opening pre header. The existing live parser scope retains exact
raw pre and actual inherited lexical mode independently. Modified/argument-only scopes preserve their actual
lexical mode and original owner but record typed ambiguity instead of inventing
a lint kind. That ambiguity conservatively persists through their descendants,
even if a later descendant spells exact `v-pre`; no extra header walk recovers it.
An exact `v-pre` on the same complete opening header resolves that scope in either
attribute order. Original structural heads on that opening template still establish
its actual registered Template kind. An inherited literal template instead carries
a typed InheritedTemplate refusal: the original producer did not resolve those
raw heads, while the registered route may introduce a synthetic L2 Template
carrier. No extra scan guesses its category. Other literal descendant kinds remain
header-known. Existing scope recovery controls actual sibling inheritance.

The same original event retains whether the header was already literal through
inherited pre before its attributes were lexed. The sealed receipt exposes this
fact so dependent rules distinguish own pre headers (whose malformed modifiers
still emit parser errors) from inherited raw headers (which emit no such errors).

Opening-end Event aux uses its original low mode bit plus disjoint optional
lint-kind bits. The existing mode accessor masks only its own bit; quote and raw
interpolation width meanings stay disjoint. Ordinary opening events retain their
old aux bytes. Token Debug/render/dump do not expose private construction facts.
Existing Event 12, Token/Option Token 40, OpenTag 144 and Element 248 byte caps,
all complete output laws, numeric instruction ceilings and source fixtures stay
unchanged. The reviewed storage inventory adds only three arena-Vec type-signature
references in parse.rs for the const-false wrapper around the existing constructor;
the same event/error/recovery vectors remain its only owned allocations. Normal constructors compile out lint retention; actual hosted full
instruction measurements must still prove the preserved ceiling costs.

The provider's ownership, exact-pre/ambiguity, classification and storage laws
are separate from a dependent selected rule consumer. Actual registered-facade
classification controls exercise the same authored kind implementation without
runtime imports from that legacy helper. The later no-autofocus/no-access-key/
no-distracting-elements family must retain complete warning/help/range/label/fix
comparisons and truthful parser-advisory refusals. No default route changes or
#6881 history admission are included. Fresh exact-head Actions, verified native
Stack membership, protected full/100 gates and actual merges decide delivery.

The provider adds 14 original L1 laws, nine complete registered-facade controls
and one Event aux/layout law. These cover original owner identity, moves and
reborrowing, absolute Unicode-prefix spans, namespaces, structural heads, exact
and inherited pre scopes, ambiguity, live recovery, incomplete framing and
ordinary constructor/output/layout preservation. They are authored controls
pending hosted Rust execution; the ten focused storage/i18n laws and both
canonical inventory checks pass locally.

The unchanged non-void self-closing source control keeps the actual registered
parser policy: `parse_diagnostics` suppresses its compatibility notice, leaving
one autofocus warning and no parser warning. Duplicate-attribute advisories
remain in the complete output. This corrects a source-audited control expectation
without changing production behavior, filtering oracle diagnostics or removing
the original input.

The previous iframe and tabindex Stack #7516 is terminal: #7506 merged as
`f9718b1b0ed8ea65cd3f8a4dff7a0ba11d10d76c`, and #7515 merged as
`b967435f47341f37512871160ab945764a62d55d` at 2026-10-03T09:56:25Z. The child
main-base Check 37112741576 passed all 31 unique native laws; its actual queue
Check 37113432095 passed full suites and instruction ceilings. Fresh origin/main
contains both actual merge commits. This new provider starts on that genuine
main, with a separate dependent consumer worktree planned.

The first actual hosted Check 37116528412 passed existing native rule laws and
production compilation, but exposed the inherited-template category mismatch and
an incorrect modified-pre control warning count. The original input and complete
registered output remain; the provider now refuses the unknown inherited category,
known own structural template heads retain Template, and the unchanged modified-pre
control checks its actual single outside warning. The source-length gate also
required splitting new test modules under its unchanged 350-line limit. Fresh
exact-head Actions remain required after these source repairs.

Check 37117544592 then exposed missing explicit integration-test module paths
after the split. Both roots now declare the exact private file paths; the
original cases, source strings and full comparisons stay unchanged. The next
hosted tooling capture required ordinary discovery rather than path attributes;
the roots now use inline modules with ordinary private child declarations, as
the previously merged native syntax laws do. No module-layout gate is bypassed.
