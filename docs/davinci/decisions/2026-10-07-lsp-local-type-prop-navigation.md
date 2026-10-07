# Local type prop navigation

Owning report: #8011 (the local `defineProps<Props>()` part).

## Decision

Retain exact own property-member spans while the existing OXC walk registers a
local alias or interface. The existing prop extraction then records that member
as the authored prop owner rather than the `defineProps` call. Inline,
parenthesized and intersection literal bodies retain their actual member
coordinates; repeated/conflicting members do not choose an arbitrary owner.
This metadata is private to script parsing and is not serialized into Croquis.
Generated TypeScript code and macro types are unchanged.

Follow Canon's existing `VueTemplatePropBinding` link in native reference and
rename queries. A public typed property access and a synthetic bare template
binding represent one authored prop but separate TypeScript symbols. The
producer-owned exact link joins those identities without a name-based source
sweep. Existing native scope admission and whole-edit coherence remain required.

## Qualification and remaining work

The complete original #8011 report and E.vue source are retained in
`tests/_fixtures/differential/lsp/type-alias-navigation/8011/`. Two parser laws
require exact alias/interface member custody for plain, withDefaults and partial
reactive-destructure calls and ambiguous intersection refusal. Six source-built
native stdio sessions retain original and CRLF variants, complete original
reference/rename vectors, exact definitions, applied whole source and empty
post-application diagnostics for the independent bare hidden prop. Original
plain tone navigation is covered separately; a destructured tone local and a
withDefaults key have additional identities not claimed by that control.

Formatting and whitespace checks pass at preparation. Exact-head source Actions,
protected checks, signed merge and public acceptance remain pending. Reference
alias chains, inherited/mapped/conditional properties, imported ownership,
shorthand reactive prop alias parity (#7996), event/slot linkage and #4075's
standard upstream Content Mapper lane remain independent obligations. This
slice does not close all of #8011 and makes no measured speed claim.

## Mixed-role qualification and delivery

The source-built `dff7005` service accepted eight unsafe mixed-role transactions
across inline/local-alias props, LF/CRLF and declaration/attribute directions.
The supplemental [same-name decision](./2026-10-07-lsp-same-name-rename.md)
records whole original receipts, per-occurrence endpoint custody, unchanged
original fixtures and mandatory full reference/rename/post-edit vectors.
Interface and withDefaults forms extend the correction's qualification without
claiming completion before new Actions and protected execution pass.

Stack #8184 delivered only parent #8166 at `2026-10-07T09:21:40Z`, commit
`8bc19f1c15233fccef229b2fa9044ac62fed8ff2`; the child results-upload timeout
left #8181 open. Its refreshed `dff7005` source is based on actual main and still
occupies native Stack position 2. Submission of the Stack prefix did not prove
atomic completion. Verify the child's fresh exact-head gates, native membership,
protected candidate and actual merge receipt before claiming delivery.
