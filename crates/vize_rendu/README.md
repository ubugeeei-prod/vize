# vize_rendu

Compatibility follows the [Rust crate support tiers](https://github.com/ubugeeei-prod/vize/blob/main/docs/content/stability.md#rust-crate-support-tiers).

Rendu is Vize's frontend-neutral render HIR. A frontend lowers its own syntax
directly into this owned, indexed representation; render backends consume it
without depending on the frontend AST that produced it.

The model covers host elements, components, slot definitions and outlets,
text, expressions, attributes, directives, conditionals, loops, fragments,
and hoist references. Every render item can retain source provenance, including
secondary spans for constructs synthesized from more than one source region.

Rendu has no dependency on Relief, Croquis, OXC, an SFC/JSX frontend, or an
Atelier backend. Its only Vize dependency is Atlas for the open `RenduProduct`
identity; Atlas does not define the data model.

`RenduBuilder` builds an owned indexed `RenduRoot`, validates all indexed edges,
rejects cycles, and infers the capabilities a backend must support. The walk API
then traverses the HIR without reconstructing or consulting a source AST.
