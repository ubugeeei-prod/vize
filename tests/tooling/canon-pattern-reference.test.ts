import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { patternFixtures, writePatternProject } from "./support/upstream/pattern-fixtures.ts";
import { check, patternRoot, workspace } from "./support/upstream/vue-language-tools.ts";

test("the pattern reference inventory preserves all upstream test sources", () => {
  const manifest = JSON.parse(
    fs.readFileSync(path.join(patternRoot, "upstream/manifest.json"), "utf8"),
  );
  let total = 0;
  for (const source of manifest.sources) {
    const directory = source.repository === "vuejs/core" ? "core" : "language-tools";
    for (const file of source.files) {
      const bytes = fs.readFileSync(path.join(patternRoot, "upstream", directory, file.path));
      assert.equal(createHash("sha256").update(bytes).digest("hex"), file.sha256, file.path);
      total++;
    }
  }
  assert.equal(total, 20);
  const { cases, sources } = patternFixtures();
  assert.equal(cases.length, 40);
  assert.equal(sources.size, 140);
  // A line-oriented import stripper used on a whole spec would also erase
  // imports inside the embedded SFC strings, invalidating exactType assertions.
  assert.match(sources.get("scopes.vue")!, /import \{ exactType \} from '\.\.\/tsc\/shared'/);
});

test(
  "all reference typechecking cases include nested and top-level templates",
  { timeout: 120_000 },
  async () => {
    const { cases, sources } = patternFixtures();
    const directory = workspace("pattern-reference-");
    writePatternProject(directory, sources);
    try {
      const diagnostics = (await check(directory)).filter((d) => d.severity === "error");
      const verified = new Set<string>();
      for (const [index, [name, , , exhaustive]] of cases.entries()) {
        for (const file of [`${index}.vue`, `root-${index}.vue`, `root-${index}-html.vue`]) {
          const actual = diagnostics.filter((d) => d.file === `patterned-templates/${file}`);
          assert.equal(
            actual.length,
            exhaustive ? 0 : 1,
            `${file} (${name}): ${JSON.stringify(actual)}`,
          );
          if (!exhaustive) {
            const diagnostic = actual[0];
            assert.match(diagnostic.message, /Non-exhaustive v-match/);
            const source = sources.get(file)!;
            const offset = source.indexOf('v-match="subject"') + 'v-match="'.length;
            const before = source.slice(0, offset).split("\n");
            assert.equal(diagnostic.line, before.length, file);
            assert.equal(diagnostic.column, before.at(-1)!.length + 1, file);
            if (name === "missing literal") assert.match(diagnostic.message, /v-when=\\?"'c'/);
            if (name === "missing tuple product")
              assert.match(diagnostic.message, /\['right', 'bottom'\]/);
          }
          verified.add(file);
        }
      }
      for (const file of sources.keys()) {
        if (verified.has(file)) continue;
        const expected = file.startsWith("duplicate-")
          ? /Duplicate pattern binding value/
          : file === "invalid.vue"
            ? /Only const pattern bindings/
            : file === "javascript-missing.vue"
              ? /Non-exhaustive v-match/
              : undefined;
        const actual = diagnostics.filter((d) => d.file === `patterned-templates/${file}`);
        assert.equal(actual.length, expected ? 1 : 0, `${file}: ${JSON.stringify(actual)}`);
        if (expected) assert.match(actual[0].message, expected);
      }
      assert.ok(
        diagnostics.every((d) => sources.has(path.posix.basename(d.file))),
        JSON.stringify(diagnostics),
      );
      const expectedMutations: Array<{ file: string; line: number }> = [];
      for (const [file, source] of sources) {
        if (!source.includes("@vue-expect-error")) continue;
        source.split("\n").forEach((line, index) => {
          if (line.includes("@vue-expect-error"))
            expectedMutations.push({ file: `patterned-templates/${file}`, line: index + 2 });
        });
        fs.writeFileSync(
          path.join(directory, "patterned-templates", file),
          source.replaceAll("@vue-expect-error", "expected error removed"),
        );
      }
      assert.equal(expectedMutations.length, 10);
      const key = (d: { file: string; line: number; column: number }) =>
        `${d.file}:${d.line}:${d.column}`;
      const original = new Set(diagnostics.map(key));
      const exposed = (await check(directory)).filter(
        (d) => d.severity === "error" && !original.has(key(d)),
      );
      const sort = (a: { file: string; line: number }, b: { file: string; line: number }) =>
        a.file.localeCompare(b.file) || a.line - b.line;
      assert.deepEqual(
        exposed.map(({ file, line }) => ({ file, line })).sort(sort),
        expectedMutations.sort(sort),
        "every removed upstream expectation must expose its original error",
      );
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
