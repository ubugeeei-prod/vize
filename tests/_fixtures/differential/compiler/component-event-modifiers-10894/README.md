Regression for https://github.com/ubugeeei-prod/vize/issues/10894 by naitokosuke.

The pinned SSR module is a compatibility control captured with published `@vizejs/native@0.587.0`; it does not demonstrate client event semantics. Client guard ordering, cancellation, bubbling, rejected keys and repeated component emits are independently asserted by `crates/vize_atelier_vapor/tests/davinci_mounted_behavior/component_events.rs`.

The original consumer is https://github.com/naitokosuke/vize-component-event-modifiers-repro at `8449f0328b52d23720f1acb1e74bba0270240c74`; its Vue build passes while the original Vize build fails.
