const { compileSfc } = require(require.resolve("@vizejs/native", { paths: [process.argv[2]] }));
const source = "<script setup>const n = 1</script><template><p>{{ n }}</p></template>";
if (!JSON.stringify(compileSfc(source, { filename: "Smoke.vue" })).includes("createElementBlock"))
  process.exitCode = 1;
