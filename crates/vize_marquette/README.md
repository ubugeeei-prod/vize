# vize_marquette

`vize_marquette` is the versioned application blueprint shared by Vize target
adapters. Like an artist's scale model, a marquette describes the environments,
routes, backends, protocols, and capabilities of the finished application
before any target-specific build begins.

Support and deprecation guarantees are defined in the
[Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

## Guarantees

- deterministic validation diagnostics with stable machine-readable codes;
- canonical serialization and fingerprints for cache and provenance keys;
- fail-closed compatibility reports for validated additive and breaking graph changes;
- a checked JSON Schema embedded in the crate;
- shared conformance fixtures for adapters implemented in other languages.

## Example

```rust
use vize_marquette::{
    ApplicationContract, Environment, EnvironmentConsumer, RuntimeFamily,
    Target,
};

let mut marquette = ApplicationContract::new("gallery");
marquette.targets.insert(Target::Web);
marquette.environments.push(Environment::new(
    "browser",
    Target::Web,
    EnvironmentConsumer::Client,
    RuntimeFamily::Browser,
));

assert!(marquette.validate().is_empty());
```

## License

MIT
