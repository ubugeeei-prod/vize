import { createHash } from "node:crypto";
import { priorRegisteredWorkflow } from "../../ci/workflow-program-history.ts";

type ReadFile = (side: "base" | "head", file: string) => Buffer | null;
const classificationWorkflows = new Set([
  "language-project-fixtures",
  "typescript-plugin",
  "typescript-provider-vsix",
]);

/** Pass authenticated pre-extraction sources to the original classification custody chain. */
export function beforeClassificationPrograms(read: ReadFile): ReadFile {
  return (side, file) => {
    const bytes = read(side, file);
    if (side !== "head" || !bytes) return bytes;
    const owner = /^\.github\/workflows\/([\w-]+)\.yml$/u.exec(file)?.[1];
    if (owner && classificationWorkflows.has(owner))
      return priorRegisteredWorkflow(owner, bytes, (file) => read(side, file));
    if (file !== "tests/tooling/language-project-classification-runtime.test.ts") return bytes;
    const previous = bytes
      .toString("utf8")
      .replace('import { archivedWorkflow } from "./support/workflow-history.ts";\n', "")
      .replace("      const predecessor = archivedWorkflow(workflow);\n", "")
      .replace(
        'predecessor.slice(predecessor.indexOf("\\njobs:"))',
        'current.slice(current.indexOf("\\njobs:"))',
      );
    const original = Buffer.from(previous);
    return createHash("sha256").update(original).digest("hex") ===
      "21fd3bb271d74b660708d988d870e1682500dcd32857a6fb16d23e5096c83fa4"
      ? original
      : bytes;
  };
}
