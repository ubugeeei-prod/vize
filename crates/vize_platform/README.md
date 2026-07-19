<h1 align="center">
  <img src="https://raw.githubusercontent.com/ubugeeei-prod/vize/main/assets/crates/vize_platform.svg" alt="vize_platform logo" width="120" height="120" /><br>
  vize_platform
</h1>

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_platform` defines the versioned, language-neutral application contract shared by Vize's web,
native, desktop, terminal, and backend surfaces. The crate owns contracts rather than runtime
implementations, so target-specific crates can consume one model and report their capabilities
through it.

## Key Entry Points

- `ApplicationContract`
- `validate_contract`
- `canonical_json`
- `compare_contracts`
- `APPLICATION_CONTRACT_JSON_SCHEMA`

## License

MIT
