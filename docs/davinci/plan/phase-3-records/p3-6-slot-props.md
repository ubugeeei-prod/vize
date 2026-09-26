# P3-6 — Computed slot outlet prop names (2026-09-26)

Ordinary `<slot :[name]="value">` props retain their S2 key expression AST in
the checked payload. The shared generator emits a reactive computed-key source
in the raw slot props' `$` list. Static groups before and after these sources
retain authored order. Static keys are camelized; computed keys keep their
runtime spelling. Slot class/style values merge into raw arrays, without DOM
normalization. The same helper corrects ordinary static-key camelization,
valueless attributes and repeated class/style values.

A static `name` or `:name` remains the slot selector. A computed key whose
authored expression is `name` is an ordinary prop. The retained lowerer now
keeps this distinction and clones the original key AST, without synthetic
JavaScript or expression reparsing.

Twelve source fixtures compare exact native and retained code/templates in
both prefix settings. They cover reference, indexed, call and concatenated
keys, both static/computed authored orders, static key normalization,
class/style merging, scoped slot aliases and valueless attributes. A checked
payload mutation snapshot changes the selector, computed key and value while
generation receives an unrelated source. Nine floor fixtures require zero
legacy walks and zero expression reparses in both prefix settings. The seven
existing allocation ceilings remain unchanged.

Three exact generated-code and complete decoded-map snapshot pairs prove
equality with the retained lane. These preserve the existing selector,
fallback and render mapping units; they do not introduce individual prop key
or value anchors. Three malformed HTML sources pin parser diagnostic code,
message and location across both lanes.

Five TS-33 tests compare sixteen mounted scenarios with the pinned official
compiler and runtime: static restoration across three key forms in both
authored orders, static/computed key normalization, raw class/style merging,
ordinary valueless props, dynamic selector changes and fallback lifetimes.
Exact traces include updates, removal, stable DOM identity and unmount.
Retained generation is proved separately by source and map equality.

Slot prop modifiers, named/computed events, object spreads, models, refs,
duplicate non-class/style keys and duplicate selector sources remain explicit
legacy surfaces. This contract does not close P3-6's complete surface or
performance acceptance.

Contract: [P3-6](p3-6.md).
