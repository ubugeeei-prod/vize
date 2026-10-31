# Windows verbatim overlay inputs

`input.json` is the source input for the Windows materialized-revision unit
`corsa_bridge::vue_document::build::tests::windows_overlays::verbatim_windows_overlay_inputs_preserve_materialized_revision_bytes`.
The test compile-includes these bytes and consumes the main SFC, equivalent
script entry, imported child SFC, support module, original alias configuration,
and all four valid/error/repair/close input states. Both child and support have
unsaved revisions; closing them restores the original disk inputs.

The input is registered through the existing no-pack corpus mechanism. Its
`batch` target is planned corpus accounting: this row has no curated diagnostic
pack, expected clean result, native adapter, or native-admission claim. The
coverage is source inspection of the decoded Vue/TypeScript inputs. Actual
Windows execution and broader native diagnostics require their own Actions
evidence.

This is separate from the logical-workspace protocol, the existing 11208 App
input, and all historical wire/source-pairs records. Those inputs and oracles
are unchanged.
