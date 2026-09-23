# vize_extension_sdk

The SDK for Vize extension-contract guests: the canonical `vize:contracts`
WIT package (`wit/`) and its released surface history (`versions/`),
its bindings, the capability-handshake constants, writers for the S1 and S2
pages the host accepts, and the runtime an import-free `no_std`
`wasm32-wasip2` guest provides itself. It depends on no Vize implementation
crate.

Contract versioning follows
[the compatibility policy](https://github.com/ubugeeei-prod/vize/blob/main/davinci-road/contracts-compat-policy.md).
Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## License

MIT
