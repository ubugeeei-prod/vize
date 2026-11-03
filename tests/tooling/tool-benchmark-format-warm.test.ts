import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";

import { measureFormat } from "../../tools/benchmarks/scripts/compare-tools-format.ts";

void test("formatters keep one warm work dir and restore the unformatted input per run", async (t) => {
  const root = mkdtempSync(join(tmpdir(), "vize-fmt-warm-"));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  const input = join(root, "in");
  mkdirSync(input);
  writeFileSync(join(input, "A.vue"), "<template><div  /></template>\n");
  writeFileSync(join(input, "vite.config.mjs"), "export default {}\n");
  const vize = join(root, "vize");
  writeFileSync(vize, "");
  const calls: { binary: string; cwd: string; cacheWarm: boolean; before: string }[] = [];
  const surface = (await measureFormat(
    input,
    ["A.vue"],
    { vizeBin: vize },
    {
      workRoot: join(root, "work"),
      resolveWorkspaceBin: () => "prettier",
      totalFileBytes: () => 1,
      copySelectedFiles: (from, to, files, extra) => {
        rmSync(to, { recursive: true, force: true });
        mkdirSync(to, { recursive: true });
        for (const file of [...files, ...extra])
          writeFileSync(join(to, file), readFileSync(join(from, file)));
      },
      runCommand: (binary, _args, { cwd }) => {
        const cache = join(cwd, "node_modules/.vize/config-cache");
        calls.push({
          binary,
          cwd,
          cacheWarm: existsSync(cache),
          before: readFileSync(join(cwd, "A.vue"), "utf8"),
        });
        // What the CLI does: fill the config cache and rewrite the input.
        mkdirSync(join(cwd, "node_modules/.vize"), { recursive: true });
        writeFileSync(cache, "cached");
        writeFileSync(join(cwd, "A.vue"), "<template><div /></template>\n");
        return { ms: 1, cpuMs: 1 };
      },
      measureVariants: async (variants) => {
        for (let run = 0; run < 3; run++) for (const variant of variants) variant.measure();
        return [];
      },
      createSurface: (value) => value,
    },
  )) as { id: string };
  assert.equal(surface.id, "fmt");
  for (const label of ["prettier", "vize-1t", "vize-max"]) {
    const runs = calls.filter((call) => call.cwd.endsWith(join("fmt", label)));
    assert.equal(runs.length, 3, label);
    assert.deepEqual(
      runs.map((call) => call.cacheWarm),
      [false, true, true],
      `${label}: only the first run fills the config cache`,
    );
    for (const call of runs) assert.equal(call.before, "<template><div  /></template>\n", label);
  }
});
