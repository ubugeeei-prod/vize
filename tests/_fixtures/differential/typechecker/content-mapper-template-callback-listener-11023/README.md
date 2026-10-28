# Template callback context beside component listeners

Copied byte-for-byte from @Q16solver's report in
`ubugeeei-prod/vize#11023`, which traces the production case to
`Esposter/Esposter@407e7cf15219405110efdd0c977bb30cb1db1644`. The clean case
preserves the reported minimal project. The authored control changes only the
callback body and must retain its mapped member diagnostic.

The generated-only TS7006/TS7053 regression was observed in Vize `v0.602.0`
at `92223735e9c41cefbfe584d7f364e4847569c92b`. Its authoritative oracle is the
raw external-mapper conformance test because the legacy batch projection
intentionally drops diagnostics without authored mappings.
