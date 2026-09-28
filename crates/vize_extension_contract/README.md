# vize_extension_contract

Transport-free records, feature negotiation and canonical page acceptance for
Vize extension worlds. Compiled-in dialects depend on this crate without
acquiring Wasmtime or a component transport. `vize_extension_host` reexports
these public modules and adds the sidecar and optional Wasmtime host.

This internal crate is experimental and unpublished. The canonical WIT and
versioned guest SDK remain in `vize_guest`; the wire ABI is unchanged.
