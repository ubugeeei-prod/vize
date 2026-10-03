# Original-Program JSX/TSX references (#6838, #6844)

The first JSX/TSX File slice records runtime references from the original
admitted Program in the existing L2 resolver traversal. Previously the File
producer refused every JSX profile, even a module containing `<Comp>{value}</Comp>`.
Explicit JSX/TSX modules now admit the bounded grammar below and resolve genuine
component and container references against their real File scopes and declarations.

## Inputs and outputs

`ProgramInput` still requires the immutable parser-owned admitted Program,
default parser options, the original checked full-file `SourceBlock`, its exact
content range, and the original module profile. A copied buffer, foreign File,
short block, parse-error observation or unambiguous profile cannot supply this
authority. The source comes from that Program once; the private `ProgramJsx`
reference source is selected from its actual profile. No caller flag, second
parse or separate identifier traversal selects JSX support.

The same statement walk invokes the same expression resolver. The result remains
the existing File binding/reference rows, original scope identities, authored
full-file spans and typed issues. JSX does not mint a framework binding or a
template node. File lookup and unit closure use the existing pending rows.

## Supported subset

- Component names are the parser's actual `JSXElementName::IdentifierReference`.
  Only the opening name contributes a read. The closing name is checked without
  publishing a duplicate reference.
- Elements and fragments admit text, boolean/string attributes with plain
  identifier names, nested admitted elements/fragments, empty expression
  containers, expression containers and spreads. Runtime expression children
  delegate directly to the existing supported resolver grammar.
- All wrappers and name leaves use checked original source spans and the existing
  depth/work budgets. Runtime references retain their actual read/write,
  shorthand and constructor roles and source order.
- The actual call, constructor, tagged-template and dynamic-import observer hooks
  run in the same traversal inside JSX expressions. Observer refusal and any
  later unsupported subtree roll back references and accepted callback rows.

Intrinsic `Identifier` tags, member tags, namespaced tags or attributes, `this`
tags and element type arguments are typed unsupported. This slice does not
guess framework/name-case rules. Unsupported nested expressions, including
TypeScript wrappers or arrow/function expressions unsupported by the delegated
walk, remain unsupported. Later intrinsic/name support needs its own real grammar
policy and tests.

The generic retained `JsExpr` resolver still refuses JSX; the Program-only source
capability does not authorize L4 to copy JSX tokens into a JS module. The Vue File
facade's JSX profile refusal stays in place. JSX transform/emission, intrinsic and
member grammar, Vue JSX/TSX exposure and product/default routing remain unfinished.
No legacy implementation is used.

## Evidence and delivery

The source starts from actual signed main `4c054d2012d4f5cf815163065ab12c3c147712b1`
through the genuine published invocation prerequisite
`35c32eb0eb5be7352881781d103e4ddf5115af5e`. Its invocation hooks and JS/TS policies
are preserved. The old explicit TSX module negative fixture is replaced by real
positive JSX/TSX File facts; the existing function refusal retains its negative
JSX Script profile rather than continuing to reject newly supported modules.

Local source-qualified checks pass 131 Rust functions: 61 whole-L2 units and
70 relevant File/resolution functions, including 16 new JSX functions. The same
original-Program profile law separately passes against frozen actual `35c` and
observes its original `InvalidProfile` refusal. Tests cover original AST callback
identity, component opening-only reads, containers/spreads/fragments, Unicode and
nonzero-origin spans, real unresolved names, unsupported controls, source/profile
authority, work bounds and transactional rollback. Whole production/unit and all
selected integration checks use warning-denying Clippy and one pinned OXC/L0/L1
metadata universe, without Cargo or rebuilt dependencies.

The new per-file storage row measures test-only transactional observer evidence
(three reference/callback vectors, comparison results and one budget input string).
Production JSX traversal adds no owned buffer. Current official source inventories,
formatting/caps and finite independent source review precede publication. These
local results do not establish hosted instruction counts, product output parity
or native acceptance rates.

The paired issue comments are drafted with this change. Delivery uses a genuine
native Stack if the invocation prerequisite is still unmerged. Fresh exact-head
required/full Actions, all unchanged instruction probes, protected queue
candidates and actual merge must be confirmed before this slice is delivered.

The publication replay preserves the reviewed Rust bytes on actual signed
Exposure merge `5cdef3d638c8e7d9252cd365ffe131950092cd45`, whose parent is the
merged Glyph/security main. The invocation prerequisite is now merged. Publish
this layer from `main` and the separately reviewed static-name layer from this
head, register both in one native Stack, and follow fresh exact-head and protected
queue checks. The earlier local executions remain evidence of the original
reviewed source; fresh hosted results qualify the replayed heads.

The first hosted replay exposed two integration-only failures. Shared JSX test
helpers now use ordinary module declarations, as required by the repository
layout gate. The Vue profile law still requires the exact Vue refusal for JSX
and TSX modules and preserves the original observations and unit receipts. Its
retained generic File now completes for those supported profiles; CommonJS and
declaration-file profiles remain incomplete. Generic File completion grants no
Vue exposure or template admission. The original failed runs remain recorded;
the corrected heads need fresh hosted checks.
