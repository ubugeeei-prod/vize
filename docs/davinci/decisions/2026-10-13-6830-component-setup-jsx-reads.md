# JSX component references in script setup

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The pinned real-project release matrix reports `SlideItem` as unused in
Douyin's `SlideVerticalInfinite.vue`, despite its JSX render expression.
The script reference pass parsed every setup block as TypeScript, so JSX
caused it to reject the program before examining identifier ownership.

Parse this existing reference pass using the setup block's declared JS, TS,
JSX or TSX language. Keep the existing lexical shadow checks and every
ordinary unused-import diagnostic. This adds no analysis stage or release
budget exception.

The differential corpus includes the unchanged pinned Douyin file and
minimal JSX/TSX reads, an unused import, and a same-named arrow parameter.
The original release binary reports both positive controls and the real
file incorrectly; the unused and shadowed controls must retain their exact
import ranges. Actions validates the native fix before protected merge.

This change addresses JSX script reads. Template value reads and baseline
scope differences remain separate work and do not count as resolved here.

The finite current-source inventory updates only this changed reference module
and its whole-manifest checksum. All 323 current paths, the 206-path union,
immutable original rows, historical laws and frames remain present. Its
physical verification provides source identity only; the native Rust Actions
and every protected check still qualify the current PR independently.
