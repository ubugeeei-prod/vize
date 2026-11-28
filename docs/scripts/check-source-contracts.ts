/** Check the source contracts before generating any published documentation. */
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

/** Exact source audits shared by docs builds and their machine-readable contract. */
export const documentationSourceContractTests = [
  "tests/tooling/docs-public-api.test.ts",
  "tests/tooling/docs-contract-metadata.test.ts",
  "tests/tooling/lib-reference-docs.test.ts",
  "tests/tooling/fresco-public-docs.test.ts",
  "tests/tooling/patina-rule-documentation-metadata.test.ts",
  "tests/tooling/docs-executable-rust.test.ts",
  "tests/tooling/compiled-docs-inputs.test.ts",
] as const;

function main(): void {
  const result = spawnSync(
    process.execPath,
    ["--test", "--test-concurrency=1", ...documentationSourceContractTests],
    { cwd: resolve(import.meta.dirname, "../.."), stdio: "inherit" },
  );
  if (result.error) throw result.error;
  process.exitCode = result.status ?? 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
