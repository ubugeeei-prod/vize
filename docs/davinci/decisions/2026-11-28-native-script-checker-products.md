# Native editor and checker script reads

`SourceSnapshot::script_query_products_with_diagnostics_from` composes Maestro's
script outline and file-rename inputs with Canon's script syntax diagnostics
after original native SFC lowering. It borrows the existing
`NativeSfcObservation`, including useful syntax in a File that cannot be admitted.
Calls, macros, or an unsupported template do not prevent a clean original script
Program from serving these syntax readers.

The checker skips its parser only for an available original Program with the
exact requested grammar, identical original source selection, and no diagnostics.
The resulting checker packet is completely empty. Nonempty diagnostics, local
holes, and differing grammars retain the existing diagnostic reader. When the
native editor Program is unavailable, the existing combined editor/checker read
keeps its shared parser schedule. Empty blocks keep their existing empty result.

The observation must borrow the captured snapshot buffer. The existing revision,
document identity, language, descriptor selection, and final freshness checks
remain in force. Identical bytes from another buffer and equal-version reopen
remain insufficient to publish a result.

The dedicated `Native script checker products` Actions workflow retains the
exact revision, compiler, host, original tracked Vue manifest, full packet
differential results, and paired allocation/time logs. The corpus compares the
complete editor response and every serialized checker diagnostic to the existing
combined reader. Controlled recovery cases also compare public SFC checker
diagnostics. Resource checks use the existing 31-pair protocol with an A/A control
and exact allocation/peak-byte limits, with native lowering both outside and
inside the measured phase on both sides.

This explicit request API does not change the production editor route, grant
native semantic or runtime admission, remove a legacy dependency, or qualify a
default switch. Original tracked-source parity and prepared-reader resource
results do not replace the full real-project qualification or its post-
implementation zero-fallback observation period.
