# Patina native IR accessor compatibility

The IR identifier rename in #13674 also removed four getters from the public
`vize_patina` facade. Patina is a compatibility-preview crate, so the
[Rust support policy](./support-policy.md) requires a deprecated compatibility
window even though the returned IR types belong to experimental crates.

| Public type                              | Deprecated getter | Canonical getter |
| ---------------------------------------- | ----------------- | ---------------- |
| `native::canonical::NativeCanonicalLint` | `file()`          | `l2ir()`         |
| `native::sfc::NativeSfcLintOwner`        | `descriptor()`    | `l1ir()`         |
| `native::sfc::NativeSfcLintOwner`        | `file()`          | `l2ir()`         |
| `native::sfc::NativeSfcTemplateOwner`    | `descriptor()`    | `l1ir()`         |

The deprecated methods forward to the canonical getters with their original
signatures and lifetimes. They borrow the same retained values and add no
owner state, parsing, or allocation.

The deprecated methods must remain available for at least one published minor
release, as defined by the support policy; patches do not complete that window.
The release notes must identify the actual introducing tag containing this
compatibility restoration. A removal proposal must identify that tag and the
later intended removal release, and demonstrate that the existing minimum
window has been satisfied. The intended removal release need not already be
published. This window is independent of the vocabulary aliases in #13707.
No `deprecated(since)` version is guessed from an unreleased workspace version.
