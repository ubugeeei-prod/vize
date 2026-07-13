# vize_module

Compatibility follows the [Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

`vize_module` is the production-neutral JavaScript/TypeScript frontend. It
owns parser-lifetime-free module facts and projects OXC's real semantic CFG to
the peer `vize_flow` representation. Raw JS/TS, SFC script blocks, and JSX/TSX
frontends can therefore share imports, exports, declarations, references, and
control flow without routing infrastructure through Croquis or Relief.

The crate depends on Atlas and Flow, but deliberately has no dependency on
Croquis, Relief, Atelier, Patina, or Canon.
