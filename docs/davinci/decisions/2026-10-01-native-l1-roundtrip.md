# Standalone native L1 roundtrip consumer

Tracked in [#6835](https://github.com/ubugeeei-prod/vize/issues/6835).

## Decision

Connect `vize dump --level l1 --roundtrip FILE` to the ordinary-library native
`vize_l1::markup::parse_component` provider. The command constructs an actual
native surface tree, renders it through the existing L1 renderer and requires
byte identity. Existing CLI output, failure exits and input-file preservation
remain unchanged. Recoverable syntax diagnostics still permit a successful
lossless roundtrip; this command is a source-fidelity check.

The provider is the preceding native GitHub Stack layer, recorded in the
[native construction decision](./2026-10-01-l1-native-surface-construction.md).
This consumer proves the provider is used outside its tests. Neither layer
claims #6835 closure or a single production lexer.

`dump --all-levels` continues to capture the selected product compilation's
executed levels and same-run provenance. It must not insert an independent native
L1 parse into that feed. Compiler products, ICE replay and reducers retain their
existing routes; native product admission and fix-history verification remain
separate work. `v-pre`, Document tree rules, custom/raw interpolation and typed
embeds remain unfinished under the provider's explicit admission contract.

## Validation and remaining work

Source-built CLI tests preserve exact output, input bytes and recoverable-error
behavior for Unicode/CRLF, empty input, unterminated quotes, complete multi-scalar
entity references, interactive recovery and the pending `v-pre` recovery law.
The provider independently checks tree fields, events, holes and ordered
diagnostics across the 42 fixtures and every UTF-8 prefix/suffix.

Require exact-head Actions and review for both Stack layers, unchanged all-100
manual instruction measurements, actual native Stack membership and ordered
positions, then protected cumulative queue checks. Do not auto-merge an
individual layer. Rebase and revalidate remaining children after any parent
squash merge; record actual merges and fresh-main ancestry on the issue.

TODO: finish the remaining L1 scope/profile contracts and typed artifacts before
proposing any compiler product route replacement. No tolerance or ceiling is
relaxed by this diagnostic-command consumer.
