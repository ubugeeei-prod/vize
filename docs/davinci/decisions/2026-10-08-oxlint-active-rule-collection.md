# Collect active Oxlint rules without configuration hints

Tracking: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).
Depends on the full licensed replay observer in #8212 through a native GitHub
Stack. The child branches from that actual source head; reference source is
pinned to the same genuine parent, never a projected main.

Both pinned Oxlint hosts, 1.78.0 and 1.86.0, invoke all selected Vize `Program`
visitors before their `Program:exit` visitors on the same Program object. The
bridge collects only those actual active visitors and snapshots their native
options. The first exit seals the collection, makes one native call and indexes
its complete diagnostic arrays. Each rule reports its own array in the original
visitor order with the existing locations, suppression and duplicate controls.
No configured severity is inferred from a preset or an optional rule hint.

Valid `override.rules` entries decide the actual activation and options. A scalar
severity resets an inherited option array; disabled rules are absent from the
collection. The optional `settings.vize.rules` API remains supported by the
manual/native bridge and cannot override the host's actual selection. Both
hosts must continue refusing unsupported `overrides[].settings` with the whole
original failure packet and zero native calls.

The collected result is keyed by physical file/settings revision plus the
complete active names/options. File contents, filename, settings, activation
and nested options changes require recomputation. Only one current collected
result lives per existing file-state cache entry; the 128-entry LRU and carrier
original-source authority checks remain in force. Traversal state is released
after its final active exit. Every exit revalidates the physical source, carrier
and settings revision before serving cached diagnostics; a change during
collection or after the first exit fails closed. These per-exit authority reads
remain part of the measured cost. Dual extracted programs retain the existing
physical-revision reporting controls.

The existing hosted qualification runs all 1,369 licensed master inputs, every
one of the 51 selected rules/options, all nine package roots, 19 scriptless
SFCs and six file exclusions. Complete collected native vectors are compared
with the unchanged per-rule reference API, including cold/warm and actual
source/configuration/option/activation/LRU changes. Both real hosts compare the
whole parent JS reference and current wrapper reports on the same authenticated
current source-built addon. The parent package is reconstructed from its exact
Git objects and built with existing dependencies; all source and bundle hashes,
commands, stdout/stderr/status and actual native arguments/results are retained.
The expected no-hint native-call totals are 69,813 for the parent reference and
1,369 for the collector. Hosted execution must authenticate the actual totals.

Original native/core/custom/parser/import/type-aware packets and their order,
multiplicity, unknown fields and status remain authoritative. Existing real-host
carrier and HTML controls continue unchanged; their exact eighteen hinted
51-rule calls are distinguished from newly batched unhinted calls in the raw
source-native recorder. No snapshots, input vectors, budgets or required checks
are weakened.

Fresh full hosted source and protected delivery are pending. Mock laws and
pure-JS host lifecycle probes provide no native or timing credit. Retain matched
before/after measurements without claiming an upstream 11-second result. Direct
SDK scriptless callbacks remain externally unavailable, and wrapper/native
coverage does not satisfy that separate contract. The two n8n-local plugins,
retired rules, full workspace configuration and installed release acceptance
remain unfinished.
