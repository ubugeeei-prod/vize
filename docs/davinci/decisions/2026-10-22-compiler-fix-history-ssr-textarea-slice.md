# Compiler fix-history SSR textarea slice — #6880

The compiler fix-history review ledger now admits its first SSR target while
retaining the ten existing SFC reviews. This bounded slice reviews the already
registered `compiler/ssr/textarea-model` fixture for fix
`63d11c41a88db4cead31dbbb512e3a47c8ac29e6`.

The authored `<textarea v-model="x"></textarea>` input, default public SSR
options, repeated historical observation, complete expected result, fix
provenance, and current source witness were already pinned. The review records
the requirement that the model value becomes escaped textarea text content and
that the complete public result retains generated code, helper preamble,
nullable map, and ordered diagnostics. No output is regenerated or inferred.

The review ledger now names every differential manifest from which it grants a
bounded review, and the generator validates each target's exact complete-result
contract before changing the fix-history index. This prevents an SSR review
from inheriting the SFC field contract or appearing without an explicit
fixture-provenance link.

Existing SSR and Vapor provenance links in the generated index now use their
fully qualified `compiler/{target}/{case}` manifest identities. This is an
identifier normalization only: it does not add fixture links, coverage, or
acceptance.

This advances bounded semantic review from ten to eleven fixture scopes. It
does not complete the textarea fix across other options, dialects, form
controls, SFC integration, hydration, or native compilation. All 626 pinned fix
subjects retain whole-fix target and dialect review pending, the semantic
denominator remains unset, native acceptance remains zero, and #6880 stays
open.
