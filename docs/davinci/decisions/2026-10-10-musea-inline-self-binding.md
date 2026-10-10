# Inline Art Self binding

Issue: [#8460](https://github.com/ubugeeei-prod/vize/issues/8460), discovered by the installed third-party [#8329](https://github.com/ubugeeei-prod/vize/issues/8329) replay.

## Observed failure

Public 0.439.0 Actions [38032119383](https://github.com/ubugeeei-prod/vize/actions/runs/38032119383) reaches the gallery and fails the original 30-second first-frame predicate. A separate macOS consumer installed the exact public npm packages and literal release `a26243855bcf81005049252a6e71b6dd55e457f1` example files. Its retained iframe documents show all six expected initial globals, one setup call and distinct document tokens, but the rendered host is an unresolved `museacomponent` element rather than `button.btn`. Its module responses return HTTP 200 without JavaScript errors.

The synthetic SFC imports `__MuseaComponent`, while inline Self expands to `MuseaComponent`. The native compiler removes the unused import and emits runtime component resolution. This is a product component-binding mismatch, not a reason to extend the readiness deadline.

## Decision and controls

Use `MuseaComponent` for both the inline synthetic import and Self expansion. Inline Art does not copy the host setup into the variant, so this binding does not collide with authored setup. Standalone Art and the existing art-module exports retain their separate scopes.

The consumed JSON fixture preserves a minimal authored host, inline variant and expected expansion outside the compiler Vue fixture inventory. The resolver control failed with the original names and passes with the aligned binding. A native parser/SFC control requires the real host import to survive and forbids unresolved runtime resolution. Actions builds the exact source native binding and gallery before rendering the real example: all six variants, disabled state, button counts, globals, original setup globals and actual Document identity are checked; failure DOM and screenshots remain artifacts.

The native browser job pins the reviewed rust-toolchain action to a commit reachable from its durable master history, with Rust 1.99.0 still explicit. The copied older stable-action pin is no longer in that action repository's live branch history and the actual code-scanning pedigree gate refuses a newly added reference; no audit exception or toolchain-version change is used.

A local diagnostic counterfactual modifying only that resolver assignment made every original public browser case pass through its final Linux provider-custody assertion. It is explicitly a macOS diagnostic with modified package bytes, not an installed public acceptance receipt. Public release completion still requires the unchanged full Linux consumer against the newly published packages. Existing source toolbar fixtures exercise generated globals with explicit component exports; they remain intact and are complemented by this actual Art/native path.
