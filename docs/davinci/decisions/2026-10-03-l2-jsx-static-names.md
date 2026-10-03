# Intrinsic and static member JSX names (#6838, #6844)

This is an own-only successor to the [bounded original-Program JSX/TSX
slice](./2026-10-03-l2-program-jsx-subset.md), whose immutable source remains
`4044f7cfa13aadd3e9e557fb03d9bcbb8d34d908`. It adds two actual OXC JSX name
variants to the same resolver traversal. The first source/public delivery does
not wait for this independently reviewable child.

## Source and reference contract

An actual `JSXElementName::Identifier` is a static intrinsic name. Its source
leaf must be nonempty with checked UTF-8 endpoints, and it contributes no
binding reference. The resolver does not test lowercase names, search for
framework globals or create a tag binding. Containers and spreads retain their
genuine runtime references.

An actual `JSXElementName::MemberExpression` is a static JSX member chain.
The same traversal descends its original `JSXMemberExpressionObject` nodes to
one authentic `IdentifierReference` root and records that opening root as a read
in the real enclosing File scope. Every member wrapper and property leaf uses
the existing checked source spans and depth/work budgets. Static properties do
not become identifier reads; no registry decides which root is a component.
The original closing name is checked through the same helper without publishing
a second reference. A lowercase root such as `ui.Panel.Button` therefore follows
the actual AST reference variant, not a casing heuristic.

The private helpers take only the original name AST, current depth and actual
opening/closing position. They add no public caller policy or admission API,
separate traversal, parse, serialization or owned production buffer. Deferred
File resolution, authentic units/scopes, original parser/source/profile
admission and the existing invocation observers remain unchanged.

## Refusals and transactions

`this` roots, namespaced tags/attributes, element type arguments and unsupported
nested runtime grammar remain typed unsupported. No TypeScript erasure is added.
Invalid intrinsic/member leaves, a late unsupported namespace, a genuine sink
refusal or an excessive member chain roll back all references and accepted
observer rows from that expression while retaining prior rows. Opening and
closing Unicode leaf laws check failures before and after descendant callbacks.

The original generic `JsExpr` route still refuses JSX. The Vue File facade still
refuses JSX profiles, and L4 JSX output, transforms, Vue exposure and product or
default routing remain unfinished. This semantic source fact does not establish
rendering, runtime access, purity, template custody or output spelling parity.
No legacy implementation or global/context fallback is used.

## Evidence and delivery

The current same-source checks pass 139 relevant Rust functions: 65 whole-L2
units and 74 File/resolution functions, including eight new static-name
functions. All prior 131 functions remain registered and pass. Their intrinsic
and member rejection examples now have genuine positive File laws; refusal
controls retain genuine `this` and late namespace cases. Whole production/unit
and all selected integration checks use warning-denying Clippy in one existing
pinned OXC/L0/ordinary-L1 universe, without Cargo or rebuilt dependencies.

The exact new intrinsic and member positive functions run RED against the frozen
`4044` library. A separate before-law verifies its real `UnsupportedSyntax`
issues, authored tag spans and retained declarations for both JSX and explicit
Module TSX. Those before executions establish the missing behavior rather than
new acceptance. Final current logs, original failed fixture checks and source
snapshots stay preserved with exact library/input hashes.

Current official source inventories, the measured test-evidence storage row,
source caps and formatting accompany this change. The new storage belongs only
to assertions and the adversarial member-depth input; production name traversal
adds no buffer. The paired issue decisions are drafted with the central L2
record in the same change.

Finite independent source review precedes publication. The child uses actual
`4044` ancestry and a verified GitHub native Stack while its prerequisite remains
open. Fresh exact-head required/full Actions, all unchanged instruction probes,
protected queue candidates and actual merge are still required. Local semantic
laws do not grant hosted performance or complete native-product acceptance.

The publication replay is a direct child of the first JSX layer replayed onto
actual signed Exposure merge `5cdef3d638c8e7d9252cd365ffe131950092cd45`.
Its reviewed Rust bytes are conserved. The first replay and this child are
registered in the same ordered native Stack; only fresh hosted results qualify
their new heads. Neither layer claims a JSX transform or product replacement.
