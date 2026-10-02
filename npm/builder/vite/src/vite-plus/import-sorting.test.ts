import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { formatSfc, type FormatOptionsNapi } from "@vizejs/native";
import { defineConfig } from "../vite-plus.ts";
import { resolveConfigExport } from "../config.ts";
import { taskConfigKey, type ConfigWithVizeTasks } from "./types.ts";

const reference = createRequire(new URL("../../../../../package.json", import.meta.url))(
  "oxfmt",
) as {
  format: (
    name: string,
    source: string,
    options: { sortImports: FormatOptionsNapi["sortImports"] },
  ) => Promise<{ code: string; errors: unknown[] }>;
};

void test("native SFC sorting matches complete script output from the pinned Oxfmt", async () => {
  const fixture = fs.readFileSync(
    new URL(
      "../../../../../crates/vize_glyph/tests/fixtures/sort-imports/UserCard.vue",
      import.meta.url,
    ),
    "utf8",
  );
  const controls: Array<[string, FormatOptionsNapi["sortImports"]]> = [
    [fixture.split("\n").slice(1).join("\n").split("</script>")[0]!, {}],
    [
      'import B from "./b";\nimport A from "project/a";\nimport C from "vue";\n',
      {
        groups: [
          "external",
          { newlinesBetween: false },
          "project",
          { newlinesBetween: true },
          ["sibling", "parent", "index"],
        ],
        newlinesBetween: false,
        customGroups: [{ groupName: "project", elementNamePattern: ["project/**"] }],
      },
    ],
    [
      'import "./z-init";\nimport "./a-init";\n// stable boundary\nimport B from "./b";\nimport A from "./a";\n',
      { partitionByComment: true },
    ],
    [
      'import A from "project/a";\nimport V from "vue";\nimport B from "project/b";\n',
      { internalPattern: ["project/"], order: "desc" },
    ],
    [
      'import B from "./b";\nimport A from "./a";\n\nimport D from "./d";\nimport C from "./c";\n',
      { partitionByNewline: true, newlinesBetween: false },
    ],
  ];
  for (const [source, sortImports] of controls) {
    const expected = await reference.format("control.ts", source, { sortImports });
    assert.deepEqual(expected.errors, []);
    const output = formatSfc(`<script setup lang="ts">\n${source}</script>\n`, {
      sortImports,
    }).code;
    assert.equal(output, `<script setup lang="ts">\n${expected.code}</script>\n`);
    assert.equal(formatSfc(output, { sortImports }).code, output);
  }
});

void test("Vite+ shares Oxfmt import sorting with native Vue formatting", async () => {
  const sorting = {
    groups: ["external", ["internal", "sibling"]],
    internalPattern: ["project/"],
    newlinesBetween: false,
    order: "desc" as const,
  };
  const result = await defineConfig(
    { fmt: { sortImports: sorting } },
    { plugin: false, tasks: false },
  )({ command: "build", mode: "production" });
  const metadata = (result as ConfigWithVizeTasks)[taskConfigKey]!;
  assert.deepEqual((await resolveConfigExport(metadata.config!)).formatter?.sortImports, sorting);
  assert.deepEqual(result.fmt?.sortImports, sorting);
  assert.ok(result.fmt?.ignorePatterns?.includes("**/*.vue"));
});

void test("explicit fmt.vize sorting overrides inherited Oxfmt settings, including false", async () => {
  for (const override of [false, { order: "desc" as const }] as const) {
    const result = await defineConfig(
      { fmt: { sortImports: { order: "asc" }, vize: { sortImports: override } } },
      { plugin: false, tasks: false },
    )({ command: "build", mode: "production" });
    const metadata = (result as ConfigWithVizeTasks)[taskConfigKey]!;
    assert.deepEqual(
      (await resolveConfigExport(metadata.config!)).formatter?.sortImports,
      override,
    );
    assert.deepEqual(result.fmt?.sortImports, { order: "asc" });
  }
});
