# CSS completion feature gate

Issue: [#8344](https://github.com/ubugeeei-prod/vize/issues/8344).

The frozen v0.438.0 full Check failed strict Clippy because
`vue_css_completions` was unused in a non-native library build. The
[actual job](https://github.com/ubugeeei-prod/vize/actions/runs/37967620221/job/113945842593)
reports the helper at `crates/vize_maestro/src/ide/completion/style.rs:15`.

## Decision

Compile `vue_css_completions` under `cfg(any(test, feature = "native"))`,
matching the neighboring script and template completion helpers. Its only
production caller is the native-only `complete_with_corsa` service; the
existing `test_vue_css_completions` also needs it. Gate the wrapper-facing
`vue_completions` re-export under the same condition. Its underlying function
remains compiled because ordinary CSS completion calls it directly.
`complete_style` remains unconditional. Completion output, feature defaults
and lint policy do not change.

## Validation and delivery

The existing non-native and glyph structural feature contract remains the
regression gate. Native compilation and the existing CSS completion test
must still pass. The independent PR requires exact-source full Check and
protected qualification before a fresh official release cut. Local focused
checks supplement Actions and do not substitute for publication evidence.

The third-party reports #8328, #8329 and #8335 remain open until the actual
release succeeds. This infrastructure correction does not establish their
publication status.
