(async () => {
  const cjs = require("@vizejs/native");
  const esm = await import("@vizejs/native");
  if (cjs.marker !== "loaded stale binding") throw new Error("CJS package root did not load");
  if (cjs.compileSfc() !== "compiled by stale binding") throw new Error("CJS export missing");
  if (esm.default !== cjs) throw new Error("ESM default export does not match CJS");
  if (esm.compileSfc !== cjs.compileSfc) throw new Error("ESM named export missing");
  console.log("package root escape hatch preserved CJS and ESM exports");
})().catch((error) => {
  console.error(error);
  process.exit(1);
});
