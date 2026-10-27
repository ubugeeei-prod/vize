// Automatic `props: true` projection (#10579) against vue-tsc running the real
// plugin of both same-version providers: the source build and published 5.4.0.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { repoRoot } from "../_helpers/realworld-patch.ts";
import { resolveTsgoBinary } from "../_helpers/realworld-typecheck.ts";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { previewPage } from "./support/router-preview/authority.ts";
import { prepareProject } from "./support/router-preview/project.ts";
import { verifyBuiltProvider } from "./support/router-preview/provider.ts";
import { processRecorder, type Observation } from "./support/router-preview/processes.ts";
import { prepareExperimentalRouterProject } from "./support/router-experimental-provider.ts";

assert.equal(process.env.GITHUB_ACTIONS, "true", "real providers run in Actions");
const receiptSha = process.env.VIZE_ROUTER_PREVIEW_RECEIPT_SHA256;
const providerDirectory = process.env.VIZE_ROUTER_PREVIEW_PROVIDER;
const sourceBinary = process.env.VIZE_TEST_BIN;
assert.ok(receiptSha && providerDirectory && sourceBinary, "same-run provider and source CLI");
const output = path.join(repoRoot, "target/differential/router-preview-593d7ad/automatic");
fs.mkdirSync(output, { recursive: true });
const [source] = resolveVizeLaunchCommand(undefined, sourceBinary, { required: true, repoRoot });
const tsgo = resolveTsgoBinary();

const base = `<script setup lang="ts">
// 自動の props 🍣
import { definePage } from 'vue-router/experimental'
const props = defineProps<{ extra?: string }>()
definePage({
  params: {
    query: { page: { parser: 'int', format: 'value', default: 1 }, tags: { parser: 'string', format: 'array' }, at: { parser: 'date', format: 'value' } },
    hash: { heading: {} },
  },
  props: true,
})
const id: number = props.id
const page: number = props.page
const tags: string[] | undefined = props.tags
const at: Date | undefined = props.at
</script>
<template><div>{{ props.id.toFixed() }} {{ heading?.length }} {{ props.at?.toISOString() }}</div></template>
`;
const declaration = "const props = defineProps<{ extra?: string }>()";
// A props function stays out: vue-tsc resolves its self-import circularly.
const controls: Array<[string, string]> = [
  ["typed", base],
  ["required-component-prop", base.replace("extra?: string", "extra?: string; required: string")],
  ["wrong-param-type", base.replace("const id: number", "const id: string")],
  ["untyped", base.replace(declaration, "const props = defineProps()")],
  [
    "hoisted-alias",
    base.replace(
      declaration,
      `interface Props { extra?: string }\n${declaration.replace("{ extra?: string }", "Props")}`,
    ),
  ],
  [
    "with-defaults",
    base.replace(
      declaration,
      "const props = withDefaults(defineProps<{ extra?: string }>(), { extra: 'x' })",
    ),
  ],
  ["props-false", base.replace("props: true,", "props: false,")],
  [
    "explicit-page-generic",
    base.replace("definePage({", "definePage<'src/pages/products/[id=int].vue'>({"),
  ],
];

type Diagnostic = [file: string, line: number, column: number, code: number];

function reference(record: Observation): Diagnostic[] {
  assert.equal(record.error, null, record.label);
  return record.stdout.split(/\r?\n/u).flatMap((line) => {
    const match = /^(.+?)\((\d+),(\d+)\): error TS(\d+): /u.exec(line);
    return match ? [[match[1], Number(match[2]), Number(match[3]), Number(match[4])]] : [];
  });
}

function native(record: Observation): Diagnostic[] {
  assert.equal(record.error, null, record.label);
  const report = JSON.parse(record.stdout) as {
    files: Array<{ file: string; diagnostics: string[] }>;
  };
  return report.files.flatMap(({ file, diagnostics }) =>
    diagnostics.flatMap((text) => {
      const match = /^error:(\d+):(\d+) \[TS(\d+)\] /u.exec(text);
      return match ? [[file, Number(match[1]), Number(match[2]), Number(match[3])]] : [];
    }),
  );
}

const provider = verifyBuiltProvider(providerDirectory, receiptSha);
const preview = prepareProject(path.join(output, "preview"), provider);
const published = await prepareExperimentalRouterProject(path.join(output, "published"));
const results: unknown[] = [];
const failures: string[] = [];
try {
  for (const [kind, root] of [
    ["preview", preview.root],
    ["published", published.root],
  ] as const) {
    const processes = processRecorder(path.join(output, kind));
    const config = JSON.parse(fs.readFileSync(path.join(root, "tsconfig.json"), "utf8"));
    for (const enabled of [true, false]) {
      config.vueCompilerOptions.plugins = enabled ? ["vue-router/volar/sfc-typed-router"] : [];
      fs.writeFileSync(path.join(root, "tsconfig.json"), `${JSON.stringify(config)}\n`);
      for (const [name, page] of controls) {
        for (const [newlineId, newline] of [
          ["lf", "\n"],
          ["crlf", "\r\n"],
        ] as const) {
          const id = `${kind}/${enabled ? "enabled" : "disabled"}/${name}/${newlineId}`;
          fs.writeFileSync(path.join(root, previewPage), page.replaceAll("\n", newline));
          const expected = reference(
            processes.run(
              `${id}/reference`,
              process.execPath,
              [
                path.join(root, "node_modules/vue-tsc/bin/vue-tsc.js"),
                "--noEmit",
                "--pretty",
                "false",
                "-p",
                "tsconfig.json",
              ],
              root,
            ),
          );
          const actual = native(
            processes.run(
              `${id}/source`,
              source,
              [
                "check",
                "src",
                "--tsconfig",
                "tsconfig.json",
                "--format",
                "json",
                "--quiet",
                "--corsa-path",
                tsgo,
              ],
              root,
            ),
          );
          results.push({ id, expected, actual });
          if (JSON.stringify(actual) !== JSON.stringify(expected)) failures.push(id);
        }
      }
    }
  }
} finally {
  fs.writeFileSync(
    path.join(output, "result.json"),
    `${JSON.stringify({ failures, results }, null, 2)}\n`,
  );
  preview.cleanup();
  published.cleanup();
}
assert.deepEqual(failures, [], "every automatic control matches vue-tsc with the real plugin");
// Both packs report 5.4.0; only the source build may change the diagnostics.
const outcome = (id: string) =>
  JSON.stringify(
    (results as Array<{ id: string; expected: unknown }>).find((entry) => entry.id === id)
      ?.expected,
  );
assert.notEqual(outcome("preview/enabled/typed/lf"), outcome("published/enabled/typed/lf"));
assert.notEqual(outcome("preview/enabled/required-component-prop/lf"), "[]");
