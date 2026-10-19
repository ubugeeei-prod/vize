import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const project = fileURLToPath(new URL("../../tsconfig.tooling-migration.json", import.meta.url));
const checker = fileURLToPath(
  new URL("../../tools/support/typescript/check-project.ts", import.meta.url),
);

void test("the native tooling checker enforces strict erasable TypeScript on its actual project", () => {
  const owned = spawnSync(process.execPath, [checker], { encoding: "utf8", shell: false });
  assert.equal(owned.error, undefined);
  assert.equal(owned.status, 0, owned.stdout + owned.stderr);
  const directory = mkdtempSync(join(tmpdir(), "vize-tooling-types-"));
  const source = join(directory, "witness.ts");
  const config = join(directory, "tsconfig.json");
  writeFileSync(join(directory, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(
    config,
    JSON.stringify({
      extends: project,
      compilerOptions: { types: [] },
      files: [source],
      include: [],
    }),
  );
  const check = (text: string) => {
    writeFileSync(source, text);
    const result = spawnSync(process.execPath, [checker, config], {
      encoding: "utf8",
      shell: false,
    });
    assert.equal(result.error, undefined);
    return result;
  };
  try {
    const correct = check("export const count: number = 1;\n");
    assert.equal(correct.status, 0, correct.stdout + correct.stderr);
    for (const [text, diagnostic] of [
      ['export const count: number = "incorrect";\n', /TS2322/],
      ["export function identity(value) { return value; }\n", /TS7006/],
      ["export enum Level { First }\n", /TS1294/],
    ] as const) {
      const invalid = check(text);
      assert.notEqual(invalid.status, 0);
      assert.match(invalid.stdout + invalid.stderr, diagnostic);
    }
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});
