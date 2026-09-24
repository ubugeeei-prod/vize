import fs from "node:fs";
import path from "node:path";

const VUE_RUNTIME_PACKAGES = ["reactivity", "runtime-core", "runtime-dom", "shared"] as const;

export function hoistVueRuntimePackages(nodeModulesDir: string): void {
  const vue = JSON.parse(fs.readFileSync(path.join(nodeModulesDir, "vue", "package.json"), "utf8"));
  const version: string = vue.version;

  for (const name of VUE_RUNTIME_PACKAGES) {
    const target = path.join(
      nodeModulesDir,
      ".pnpm",
      `@vue+${name}@${version}`,
      "node_modules",
      "@vue",
      name,
    );
    if (!fs.existsSync(target)) {
      throw new Error(`Missing @vue/${name}@${version} beside fixture Vue`);
    }

    const link = path.join(nodeModulesDir, "@vue", name);
    const existing = fs.lstatSync(link, { throwIfNoEntry: false });
    if (existing?.isSymbolicLink()) {
      try {
        if (fs.realpathSync(link) === fs.realpathSync(target)) continue;
      } catch {
        // Replace a link to a version removed by a later fixture install.
      }
      fs.unlinkSync(link);
    } else if (existing) {
      const installed = JSON.parse(fs.readFileSync(path.join(link, "package.json"), "utf8"));
      if (installed.version !== version) {
        throw new Error(`Fixture @vue/${name}@${installed.version} conflicts with vue@${version}`);
      }
      continue;
    }

    fs.mkdirSync(path.dirname(link), { recursive: true });
    fs.symlinkSync(target, link, process.platform === "win32" ? "junction" : "dir");
  }
}

export function patchPnpmMinimumReleaseAgeExclude(
  workspacePath: string,
  packageName: string,
): void {
  const source = fs.readFileSync(workspacePath, "utf8");
  if (source.includes(packageName)) return;

  const entry = `  - '${packageName}'\n`;
  const nextSource = source.includes("minimumReleaseAgeExclude:\n")
    ? source.replace("minimumReleaseAgeExclude:\n", `minimumReleaseAgeExclude:\n${entry}`)
    : `${source.trimEnd()}\nminimumReleaseAgeExclude:\n${entry}`;
  fs.writeFileSync(workspacePath, nextSource);
}
