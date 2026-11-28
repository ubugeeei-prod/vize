import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const packageRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const budgets = [
  { entry: "@vizejs/marquette", file: "dist/index.mjs", maximumGzipBytes: 1024 },
  {
    entry: "@vizejs/marquette/adapter",
    file: "dist/adapter.mjs",
    maximumGzipBytes: 4096,
  },
  {
    entry: "@vizejs/marquette/validate",
    file: "dist/validate.mjs",
    maximumGzipBytes: 3072,
  },
  {
    entry: "@vizejs/marquette/native-build",
    file: "dist/native-build.mjs",
    maximumGzipBytes: 1024,
  },
  {
    entry: "@vizejs/marquette/native-build/validate",
    file: "dist/native-build-validate.mjs",
    maximumGzipBytes: 3072,
  },
  {
    entry: "@vizejs/marquette/native-build/admission",
    file: "dist/native-build-admission.mjs",
    maximumGzipBytes: 8192,
    includeDependencies: true,
  },
  {
    entry: "@vizejs/marquette/navigation",
    file: "dist/navigation.mjs",
    maximumGzipBytes: 2048,
  },
  { entry: "@vizejs/marquette/test-run", file: "dist/test-run.mjs", maximumGzipBytes: 1024 },
  {
    entry: "@vizejs/marquette/test-run/validate",
    file: "dist/test-run-validate.mjs",
    maximumGzipBytes: 4096,
  },
  {
    entry: "@vizejs/marquette/test-run/canonical",
    file: "dist/test-run-canonical.mjs",
    maximumGzipBytes: 2048,
  },
  {
    entry: "@vizejs/marquette/test-run/admission",
    file: "dist/test-run-admission.mjs",
    maximumGzipBytes: 3072,
  },
  {
    entry: "@vizejs/marquette/test-run/check",
    file: "dist/test-run-check.mjs",
    maximumGzipBytes: 3072,
  },
  {
    entry: "@vizejs/marquette/test-run/transition",
    file: "dist/test-run-transition.mjs",
    maximumGzipBytes: 4096,
  },
];

for (const { entry, file, maximumGzipBytes, includeDependencies } of budgets) {
  const files = includeDependencies ? runtimeClosure(file) : [file];
  const gzipBytes = files.reduce(
    (total, runtimeFile) =>
      total +
      gzipSync(fs.readFileSync(path.join(packageRoot, runtimeFile)), { level: 9 }).byteLength,
    0,
  );

  if (gzipBytes > maximumGzipBytes) {
    throw new Error(
      `${entry} gzip size ${gzipBytes} bytes exceeds ${maximumGzipBytes} byte budget`,
    );
  }

  console.log(
    JSON.stringify({
      entry,
      gzipBytes,
      maximumGzipBytes,
      ...(includeDependencies ? { runtimeFiles: files } : {}),
    }),
  );
}

/** Counts each emitted runtime dependency once, including shared bundle chunks. */
function runtimeClosure(entry: string): string[] {
  const files = new Set<string>();
  function visit(file: string): void {
    if (files.has(file)) return;
    files.add(file);
    const source = fs.readFileSync(path.join(packageRoot, file), "utf8");
    for (const match of source.matchAll(
      /\b(?:from\s*|import\s*(?:\(\s*)?)["'](\.\/[^"']+\.mjs)["']/g,
    )) {
      visit(path.join(path.dirname(file), match[1]!));
    }
  }
  visit(entry);
  return [...files].sort();
}
