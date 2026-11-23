const required = require("@vizejs/native");
(async () => {
  const imported = await import("@vizejs/native");
  const importedNative = imported.default ?? imported;
  for (const [label, native] of [
    ["require", required],
    ["import", importedNative],
  ]) {
    if (typeof native.compileSfc !== "function")
      throw new Error(`compileSfc missing from ${label} smoke`);
    const result = native.compileSfc(
      '<template><div>{{ msg }}</div></template><script setup lang="ts">const msg: string = "ok";</script>',
      { filename: "Smoke.vue", isTs: true },
    );
    if (
      !result ||
      result.errors.length > 0 ||
      typeof result.code !== "string" ||
      result.code.length === 0
    ) {
      throw new Error(`compileSfc ${label} runtime smoke failed`);
    }
  }
})().catch((error) => {
  console.error(error);
  process.exit(1);
});
