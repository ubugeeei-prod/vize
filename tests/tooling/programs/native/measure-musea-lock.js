import { measureMusea } from "../../../../tools/benchmarks/scripts/musea.mjs";

try {
  await measureMusea({ root: process.env.VIZE_TEST_ROOT, files: 2, runs: 1, warmups: 0 });
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
}
