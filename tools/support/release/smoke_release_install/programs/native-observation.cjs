const native = require("@vizejs/native");
const result = native.compileSfc(
  '<template><div>{{ msg }}</div></template><script setup lang="ts">const msg: string = "ok";</script>',
  { filename: "Smoke.vue", isTs: true },
);
process.stdout.write(JSON.stringify(result));
