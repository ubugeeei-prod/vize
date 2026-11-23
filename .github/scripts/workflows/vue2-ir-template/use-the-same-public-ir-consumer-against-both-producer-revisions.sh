#!/usr/bin/env bash
test "$(git -C .vue2-ir-recipe rev-parse HEAD)" = "$RECIPE_SHA"
cp .vue2-ir-recipe/davinci/vize_l4/examples/vue2_ir_template.rs davinci/vize_l4/examples/
cp -R .vue2-ir-recipe/davinci/vize_l4/examples/vue2_ir_template_lexical davinci/vize_l4/examples/
cp .vue2-ir-recipe/tests/tooling/native-vue2-ir-*.mjs tests/tooling/
# The pinned `before` source predates the one Vue owner in L1; its
# copy of the consumer names the Vue 2 surface as that source does.
if [ "$VIZE_VUE2_IR_BASELINE" = 1 ]; then
  sed -i 's/vize_l1::framework::vue::version::v2::/vize_l1::dialect::vue2::/g' davinci/vize_l4/examples/vue2_ir_template.rs
fi
