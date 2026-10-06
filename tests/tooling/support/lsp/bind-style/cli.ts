import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

export function compareCliFixes(
  binary: string,
  fixture: string,
  output: string,
  rows: Array<Record<string, unknown>>,
  save: () => void,
  prepareProviders: (project: string) => unknown,
) {
  const project = fs.mkdtempSync(path.join(output, "cli-project-"));
  for (const name of ["Parent", "Child"])
    fs.copyFileSync(path.join(fixture, `${name}.vue.txt`), path.join(project, `${name}.vue`));
  for (const name of ["vize.config.json", "tsconfig.json"])
    fs.copyFileSync(path.join(fixture, name), path.join(project, name));
  rows.push({
    method: "original CLI provider preparation",
    project,
    providers: prepareProviders(project),
  });
  save();
  const original = fs.readFileSync(path.join(project, "Parent.vue"), "utf8");
  const expected = original
    .replace('v-bind:title="title"', ":title")
    .replace(':title="title"', ":title");
  for (let index = 0; index < 2; index++) {
    const argv = ["lint", "--fix", "Parent.vue"];
    const process = spawnSync(binary, argv, { cwd: project, timeout: 30000 });
    const source = fs.readFileSync(path.join(project, "Parent.vue"), "utf8");
    fs.writeFileSync(path.join(project, `stdout-${index}.bin`), process.stdout ?? Buffer.alloc(0));
    fs.writeFileSync(path.join(project, `stderr-${index}.bin`), process.stderr ?? Buffer.alloc(0));
    rows.push({
      method: "original CLI --fix",
      argv,
      project,
      original,
      source,
      iteration: index + 1,
      status: process.status,
      signal: process.signal,
      error: process.error?.message ?? null,
    });
    save();
    assert.equal(process.error, undefined);
    assert.equal(process.signal, null);
    assert.equal(process.status, 0);
  }
  assert.equal(fs.readFileSync(path.join(project, "Parent.vue"), "utf8"), expected);
  assert.equal(
    fs.readFileSync(path.join(project, "Child.vue"), "utf8"),
    fs.readFileSync(path.join(fixture, "Child.vue.txt"), "utf8"),
  );
}
