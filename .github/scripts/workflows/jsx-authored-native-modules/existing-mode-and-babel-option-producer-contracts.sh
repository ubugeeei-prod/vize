#!/usr/bin/env bash
status=0
for target in compile ssr vapor_ssr babel_is_custom_element babel_merge_props babel_v_slots babel_compat_oracle babel_tag_classification compat_mode control_flow parity_vdom slots style v_slots vdom authored_modules pragma v_model_argument v_slots_forwarding adversarial_snapshots vapor parity_vapor parity_tsx ordered_spread_props spread_child; do
  cargo test --locked --profile ci -p vize_atelier_jsx --test "$target" || status=1
done
exit "$status"
