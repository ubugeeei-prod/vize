import { existsSync, readdirSync } from "node:fs";
import path from "node:path";

export type PklProduct = { source: string; target: string; kind: "yaml" | "shell" | "package" };

const areas = ["workflows", "actions", "repository", "scripts", "package-tasks"] as const;

/**
 * Every file the repository generates from Pkl, keyed by its repository-relative target.
 * `source` is the absolute path of its Pkl module, under `tools/config/pkl` or `tools/ci/workflows`.
 */
export function listPklProducts(root: string): PklProduct[] {
  const sources = path.join(root, "tools/config/pkl");
  const products: PklProduct[] = [];
  for (const area of areas) {
    const directory = path.join(sources, area);
    if (!existsSync(directory)) continue;
    for (const entry of readdirSync(directory, { recursive: true, withFileTypes: true })) {
      if (!entry.isFile() || !entry.name.endsWith(".pkl")) continue;
      const source = path.join(entry.parentPath, entry.name);
      const name = path.basename(entry.name, ".pkl");
      let target = path.relative(directory, source).slice(0, -4).split(path.sep).join("/");
      if (area === "workflows") target = `.github/workflows/${name}.yml`;
      if (area === "actions") target = `.github/actions/${target}/action.yml`;
      if (area === "package-tasks")
        target = path.posix.join(path.posix.dirname(target), "scripts/package-tasks.cjs");
      products.push({
        source,
        target,
        kind: area === "scripts" ? "shell" : area === "package-tasks" ? "package" : "yaml",
      });
    }
  }
  // Each workflow owns tools/ci/workflows/<file stem>/: workflow.pkl, jobs/ and scripts/.
  const ciWorkflows = path.join(root, "tools/ci/workflows");
  for (const workflow of existsSync(ciWorkflows) ? readdirSync(ciWorkflows) : []) {
    const home = path.join(ciWorkflows, workflow);
    products.push({
      source: path.join(home, "workflow.pkl"),
      target: `.github/workflows/${workflow}.yml`,
      kind: "yaml",
    });
    const scripts = path.join(home, "scripts");
    for (const name of existsSync(scripts) ? readdirSync(scripts) : []) {
      if (!name.endsWith(".pkl")) continue;
      const target = path.relative(root, path.join(scripts, name.slice(0, -4)));
      products.push({
        source: path.join(scripts, name),
        target: target.split(path.sep).join("/"),
        kind: "shell",
      });
    }
  }
  return products.sort((left, right) => left.target.localeCompare(right.target, "en"));
}

/** Repository-relative paths of the files generated from Pkl. */
export function pklGeneratedTargets(root: string): Set<string> {
  return new Set(listPklProducts(root).map(({ target }) => target));
}
