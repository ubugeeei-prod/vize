# Selected-rule batching in the Oxlint bridge

Owning issue: [#8144](https://github.com/ubugeeei-prod/vize/issues/8144).

The read-only [n8n adoption spike](https://github.com/n8n-io/n8n/pull/40393)
reports 51 native calls for each of 1,127 SFCs under the incremental preset.
Its rule map includes native options, so batching names alone would leave some
rules on a separate path.

`settings.vize.rules` is an optional batching hint: either native/plugin rule
names or an Oxlint rule map. Plugin-prefixed maps select only `vize/` entries;
native maps also support the canonical `css/` namespace. A map carries supported rule options into the
same native call. Oxlint retains reporting, severity, and override authority;
rules omitted from the hint or using different options take the existing
individual path. Explicit preset gates retain their existing meaning. When
`extends` drops runtime settings, an absent preset uses incremental native
execution so Oxlint's explicitly activated rules remain authoritative; the
helper's default rule bundle stays general-recommended. Vite+ helpers
derive the hint from their final merged Vize rule map.

The existing 128-file LRU and exact physical-source revision guard remain.
Selection names and canonical native options participate in the cache key.
Parsed selections use weak references and an authored JSON revision check,
so even an in-place settings edit cannot reuse stale native results. A mixed
type-aware selection enables that lane eagerly and keeps runtime failures
reported once.

The fixed 51-rule map comes from upstream head
`aa173be0c65c0646a7fcec32d2c18e1eaacbc8ff`. An authored SFC corpus witness
compares complete selected native diagnostics with individual rule runs.
The full 51 by 1,127 bridge workload guards exactly 1,127 native calls;
this is a call-count reduction, not a claim about n8n end-to-end lint time.

Local mock-backed bridge controls pass. Current native output qualification,
exact-head Actions, protected merge gates, and installed release verification
remain required. Other n8n adoption gaps and native Davinci migration are
outside this slice. No upstream state is changed.
