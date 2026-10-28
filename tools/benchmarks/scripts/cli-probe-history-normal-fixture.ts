/** Public original graph fixtures for optimized local probe comparisons. */
import assert from "node:assert/strict";
import path from "node:path";

export const normalProbeSizes = [10, 100, 1000];
export const normalProbeGroups = ["relative", "absolute", "mixed"];

export function normalProbeFixtureFiles(fixture: string): Record<string, string> {
  const sizes = normalProbeSizes,
    groups = normalProbeGroups;
  const files: Record<string, string> = {};
  const add = (file: string, text: string) => {
    assert.equal(files[file], undefined);
    files[file] = text;
  };
  for (const group of groups)
    for (const size of sizes) {
      const prefix = group + "/" + size;
      add(
        prefix + "/tsconfig.json",
        JSON.stringify({
          compilerOptions: { baseUrl: ".", paths: { "@probe/*": ["missing/*", "real/*"] } },
        }),
      );
      for (let start = 0; start < size; start += 50) {
        const packageRoot = prefix + "/p" + String(start / 50).padStart(4, "0"),
          imports: string[] = [];
        add(
          packageRoot + "/package.json",
          JSON.stringify({ name: "normal-probe-" + start, types: "entry.ts" }),
        );
        for (let i = start; i < Math.min(size, start + 50); i++) {
          const leaf = "leaf" + String(i).padStart(4, "0");
          let specifier: string;
          if (group === "mixed" && i % 3 === 0) {
            add(prefix + "/real/" + leaf + ".ts", "export const value: number = 1;\n");
            specifier = "@probe/" + leaf;
          } else {
            const base = i % 4 === 1 ? "folder" + i : leaf;
            const existing = i % 4 !== 2;
            if (existing) {
              add(
                packageRoot + "/" + base + (i % 4 === 1 ? "/index.ts" : ".ts"),
                "export const value: number = 1;\n",
              );
              if (i % 4 !== 1)
                add(
                  packageRoot + "/" + base + ".js",
                  "export const value = 'wrong runtime candidate';\n",
                );
            }
            const target = base + (i % 4 === 1 ? "" : ".js");
            specifier =
              group === "absolute" || (group === "mixed" && i % 3 === 1)
                ? path.join(fixture, packageRoot, target)
                : "./" + target;
          }
          imports.push("import " + JSON.stringify(specifier) + ";");
        }
        add(packageRoot + "/entry.ts", imports.join("\n") + "\nexport {};\n");
      }
    }
  return files;
}

export const normalProbeLaws = [
  "untracked_probes_keep_typescript_precedence_and_omit_candidate_histories",
  "untracked_path_mode_retains_complete_alias_probe_histories",
  "untracked_graphs_keep_exact_work_cycles_and_fresh_missing_to_created_inputs",
  "tracked_graph_control_still_retains_missing_candidates_and_real_sources",
  "cross_program_alias_agreement_keeps_equal_winners_with_distinct_proof_paths_apart",
];
