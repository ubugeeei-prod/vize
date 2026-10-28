import {
  typeProofs,
  exposedChild,
  genericChild,
  templateRefAdmission,
  mutatedAdmission,
} from "./type-admission-fixtures.ts";
import { templateRefEdgeAdmission } from "./type-admission-ref-edges.ts";

export function admissionFiles(stage: "valid" | "invalid" | "poisoned") {
  const files: Record<string, string> = {
    "proofs.ts": typeProofs,
    "Child.vue": exposedChild,
    "Generic.vue": genericChild,
  };
  const expected: { file: string; line: number; column: number; code: number }[] = [];
  for (const row of [...templateRefAdmission, ...templateRefEdgeAdmission]) {
    for (const [index, mutation] of row.mutations.entries()) {
      const file = `${row.name}-${index + 1}.vue`;
      const probe =
        stage === "poisoned"
          ? {
              name: "proof-liveness",
              before: "Assert<true>;",
              after: "Assert<false>;",
              token: "false",
              code: 2344,
            }
          : mutation;
      const { source, line, column, code } = mutatedAdmission(row.source, probe);
      files[file] = stage === "valid" ? row.source : source;
      if (stage !== "valid") expected.push({ file, line, column, code });
    }
  }
  return { files, expected };
}
