import { createHash } from "node:crypto";
import {
  isCompilerProjectCase,
  validateCompilerProjectCase,
} from "../../../../tests/differential/compiler-project.ts";

interface PinnedInput {
  path?: string;
  role?: string;
  sha256?: string;
}
interface CompilerCase {
  id?: string;
  targets?: unknown;
  adapters?: { legacy?: unknown; native?: unknown; reasons?: { native?: unknown } };
  comparison?: { contract?: unknown; requiredFacets?: unknown };
  reference?: { path?: string; sha256?: string; source?: unknown };
}
type ReadFile = (side: "base" | "head", path: string) => Buffer | null;
const COMPILER_FACETS = ["code", "css", "errors", "warnings", "macro_artifacts"];
function requirePinned(readFile: ReadFile, path: string, expected: string, label: string): void {
  const bytes = readFile("head", path);
  if (bytes === null || createHash("sha256").update(bytes).digest("hex") !== expected)
    throw new Error(`missing or mismatched pinned ${label}: ${path}`);
}

// Compiler cases compare whole SSR modules, so they also pin the expected module.
export function checkCompilerCase(
  entry: CompilerCase,
  root: string,
  files: PinnedInput[],
  readFile: ReadFile,
): void {
  const reference = entry.reference;
  const expectedPath = `${root}/module.expected.txt`;
  if (isCompilerProjectCase(entry)) validateCompilerProjectCase(entry);
  else if (
    entry.id !== `compiler/sfc/${root}` ||
    JSON.stringify(entry.targets) !== JSON.stringify(["ssr"]) ||
    files.length !== 1 ||
    files[0]?.path !== "Layout.vue.txt" ||
    files[0]?.role !== "entry" ||
    entry.adapters?.legacy !== "compiler-cli-ssr-v1" ||
    entry.adapters?.native !== null ||
    typeof entry.adapters?.reasons?.native !== "string" ||
    entry.comparison?.contract !== "compiler-ssr-module-bytes-v1" ||
    JSON.stringify(entry.comparison?.requiredFacets) !== JSON.stringify(COMPILER_FACETS) ||
    reference?.path !== expectedPath ||
    !/^[a-f0-9]{64}$/.test(reference.sha256 ?? "") ||
    typeof reference.source !== "string" ||
    reference.source.length === 0
  ) {
    throw new Error(`invalid pinned compiler reference: ${entry.id}`);
  }
  requirePinned(
    readFile,
    `tests/_fixtures/differential/compiler/${expectedPath}`,
    reference?.sha256 ?? "",
    "compiler reference",
  );
}
