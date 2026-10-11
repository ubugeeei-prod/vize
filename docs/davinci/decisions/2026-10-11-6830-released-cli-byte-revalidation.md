# Revalidate the complete released CLI at every consumer boundary

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), with the
published typed-default consumer tracked in
[#8564](https://github.com/ubugeeei-prod/vize/issues/8564).
This supersedes only the metadata-based intermediate binary shortcut in the
[original consumer](./2026-10-11-8564-released-typed-defaults-consumer.md).
Its immutable release, fixture and whole-process expectations stay in force.

Protected candidate `1b1943` for atomic
[#8593](https://github.com/ubugeeei-prod/vize/pull/8593) failed
[Check 38118165901](https://github.com/ubugeeei-prod/vize/actions/runs/38118165901).
The original custody test changed byte 80 of the same-size inert ELF file,
then expected the unchanged `/identity changed/` refusal. It instead retained
`Missing expected exception` at line 245. Full original job output has
1,430,331 bytes and SHA256
`722f642cde916af9f49ef80c61b6513725cf72430bf057340636f61abaa4fda6`.
The queue candidate was removed; that execution remains a genuine failure.

The helper cached filesystem identity and hashed only at acquisition and final
completion. The archived source and failure show its intermediate metadata
comparison accepted the rewritten bytes. Numeric hosted stat fields were not
captured, so they supply no separate measured metadata packet. Timestamp
resolution cannot establish complete file identity.

Keep every existing path, regular-file, size, mode and metadata refusal, and
also compare the complete file SHA256 to the authenticated archived member on
every call. The version process and each original CLI case verify bytes before
and after execution; final completion does too. No optional hash shortcut
remains. This adds repeated file reads to the acceptance consumer, without
changing the product pipeline or weakening any original gate.

A deterministic control freezes the real file's first BigInt stat snapshot in
a test-only metadata provider. It rewrites real same-size file bytes at offset
80 and at the last byte, retains every checked metadata field, and requires
the original refusal. Restoring the original bytes succeeds. The file is never
executed. The control fails against the original helper and passes after the
fix without sleeps, timestamp writes or changes to the original ten laws.

Prepare this bounded child privately. After the actual packing fix merges,
replay the preserved atomic parent once, then use its exact head as the native
Stack base. Require fresh source gates and one protected prefix admission.
Published GNU acceptance remains unrun until a new official release publishes;
neither this source correction nor the retired v0.445.0 supplies public credit.
