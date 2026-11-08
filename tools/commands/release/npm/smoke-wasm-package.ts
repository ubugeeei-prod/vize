import { runEntrypoint } from "../../../support/command-runtime.ts";
import { smokeWasmPackage } from "../../../support/compat/npm/smoke-wasm-package.ts";

runEntrypoint(async () => {
  const packageDir = process.argv[2];
  if (!packageDir) {
    throw new Error("Usage: node tools/commands/release/npm/smoke-wasm-package.ts <npm/wasm>");
  }
  await smokeWasmPackage(packageDir);
  console.log("@vizejs/wasm package smoke passed");
});
