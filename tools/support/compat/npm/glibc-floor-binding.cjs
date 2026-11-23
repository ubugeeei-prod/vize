const binding = require(process.argv[2]);
const result = binding.compileSfc(
  '<template><div>{{ msg }}</div></template><script setup lang="ts">const msg: string = "ok";</script>',
  { filename: "Smoke.vue", isTs: true },
);
if (!result || result.errors.length > 0 || !result.code) throw new Error("compileSfc smoke failed");
console.log("loaded " + process.argv[2] + " and compiled an SFC");
