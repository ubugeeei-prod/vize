import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  check,
  compareIdentity,
  diagnosticIdentity,
  workspace,
} from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

// A generic component forwards what its root is instantiated to: the root's
// props and listeners are read in terms of the component's own type
// parameters, so a parent's listener sees the type its own props chose.

const root = `<script setup lang="ts" generic="T">
defineProps<{ foo?: T; label: string }>();
defineEmits<{ foo: [value: T]; close: [] }>();
</script>
<template><span /></template>`;

const forwarding = `<script setup lang="ts" generic="T">
import Root from './Root.vue';
defineProps<{ bar?: T }>();
</script>
<template><Root :foo="bar" label="root" /></template>`;

function project(files: Record<string, string>, options: Record<string, boolean> = {}): string {
  const directory = workspace("generic-fallthrough-roots-");
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        module: "ESNext",
        moduleResolution: "Bundler",
      },
      include: ["*.vue"],
      vueCompilerOptions: { strictTemplates: true, fallthroughAttributes: true, ...options },
    }),
  );
  for (const [name, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(directory, name), source);
  }
  return directory;
}

async function compare(directory: string, expected: string[]): Promise<void> {
  const oracle = vueTscDiagnostics(directory).sort(compareIdentity);
  assert.deepEqual(
    oracle.map((d) => `${d.file}:${d.line}:${d.column} TS${d.code}`),
    expected,
  );
  assert.deepEqual((await check(directory)).map(diagnosticIdentity).sort(compareIdentity), oracle);
}

const parent = (usage: string) => `<script setup lang="ts">
import Forwarding from './Forwarding.vue';
const takesString = (value: string) => value;
const takesNumber = (value: number) => value;
</script>
<template>${usage}</template>`;

test("a generic component forwards its root's listeners at its own instantiation", async () => {
  const directory = project({
    "Root.vue": root,
    "Forwarding.vue": forwarding,
    "App.vue": parent(`
  <Forwarding bar="text" @foo="(value) => takesString(value)" @close="() => {}" />
  <Forwarding :bar="1" @foo="(value) => takesNumber(value)" :foo="2" />`),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent(`
  <Forwarding bar="text" @foo="(value) => takesNumber(value)" />
  <Forwarding :bar="1" @missing="() => {}" />
  <Forwarding :bar="1" foo="not a number" />`),
    );
    await compare(directory, ["App.vue:7:55 TS2345", "App.vue:8:25 TS2353", "App.vue:9:24 TS2322"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("forwarded props stay next to the slots a generic component infers", async () => {
  const directory = project({
    "Root.vue": root,
    "Forwarding.vue": forwarding.replace(
      '<Root :foo="bar" label="root" />',
      '<Root :foo="bar" label="root"><slot name="item" :item="bar" /></Root>',
    ),
    "App.vue": parent(`
  <Forwarding bar="text" @foo="(value) => takesString(value)">
    <template #item="{ item }">{{ takesString(item!) }}</template>
  </Forwarding>`),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent(`
  <Forwarding bar="text" @foo="(value) => takesNumber(value)">
    <template #item="{ item }">{{ takesNumber(item!) }}</template>
  </Forwarding>`),
    );
    await compare(directory, ["App.vue:7:55 TS2345", "App.vue:8:47 TS2345"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("a generic component forwards to every root of a conditional chain", async () => {
  const directory = project({
    "Root.vue": root,
    "Other.vue": `<script setup lang="ts" generic="T">
defineProps<{ other?: T }>();
defineEmits<{ other: [value: T[]] }>();
</script>
<template><span /></template>`,
    "Forwarding.vue": `<script setup lang="ts" generic="T">
import Root from './Root.vue';
import Other from './Other.vue';
defineProps<{ bar?: T; flip?: boolean }>();
</script>
<template><Root v-if="flip" :foo="bar" label="root" /><Other v-else :other="bar" /></template>`,
    "App.vue": parent(`
  <Forwarding bar="text" @foo="(value) => takesString(value)" @other="(values) => takesString(values[0]!)" />`),
  });
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent(`
  <Forwarding bar="text" @other="(values) => takesNumber(values[0]!)" />`),
    );
    await compare(directory, ["App.vue:7:58 TS2345"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("a generic component asks its parent for the required props its root leaves unbound", async () => {
  const directory = project(
    {
      "Root.vue": `<script setup lang="ts">
defineProps<{ foo?: string; label: string }>();
defineEmits<{ close: [reason: string] }>();
</script>
<template><span /></template>`,
      "Forwarding.vue": `<script setup lang="ts" generic="T extends string">
import Root from './Root.vue';
defineProps<{ bar?: T }>();
</script>
<template><Root :foo="bar" /></template>`,
      "App.vue": parent(`
  <Forwarding bar="text" label="from the parent" @close="(reason) => takesString(reason)" />`),
    },
    { checkRequiredFallthroughAttributes: true },
  );
  try {
    await compare(directory, []);
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      parent(`
  <Forwarding bar="text" />
  <Forwarding bar="text" label="from the parent" @close="(reason) => takesNumber(reason)" />`),
    );
    await compare(directory, ["App.vue:7:4 TS2345", "App.vue:8:82 TS2345"]);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

async function captureCase(failures: Error[], name: string, run: () => Promise<void>) {
  try {
    await run();
  } catch (cause) {
    failures.push(new Error(name, { cause }));
  }
}

test("a non-generic listener retains typed root props and strict excess checks", async () => {
  const failures: Error[] = [];

  const directory = project(
    {
      "Root.vue": `<script setup lang="ts">
defineProps<{ bar?: string }>();
defineEmits<{ foo: [value: string] }>();
</script><template><span /></template>`,
      "Forwarding.vue": `<script setup lang="ts">
import Root from './Root.vue';
</script><template><Root /></template>`,
      "App.vue": parent('<Forwarding bar="text" @foo="(value) => takesString(value)" />'),
    },
    { checkUnknownProps: true },
  );
  try {
    for (const listener of ["", ' @foo="(value) => takesString(value)"']) {
      for (const [name, attributes, code] of [
        ["valid", 'bar="text"', undefined],
        ["wrong-bar", ':bar="1"', 2322],
        ["unknown", 'bar="text" definitelyNotAProp="wrong"', 2353],
      ] as const) {
        await captureCase(
          failures,
          `${name}-${listener ? "listener" : "no-listener"}`,
          async () => {
            const source = parent(`<Forwarding ${attributes}${listener} />`);
            fs.writeFileSync(path.join(directory, "App.vue"), source);
            const actual = await check(directory, ["--show-virtual-ts"], {
              name: `${name}-${listener ? "listener" : "no-listener"}`,
              sources: ["App.vue", "Root.vue", "Forwarding.vue", "tsconfig.json"],
            });
            const expected = vueTscDiagnostics(directory);
            assert.deepEqual(
              expected.map((d) => d.code),
              code ? [code] : [],
            );
            // Canonical assembly deduplicates equal findings from the same authored
            // owner. Unknown schema keys remain the canonical checker's policy;
            // the inference input must not impose its own excess-prop check.
            assert.deepEqual(
              actual.map((d) => d.code),
              code ? [code] : [],
            );
            const line = source.split("\n")[5];
            const attribute = name === "unknown" ? 'definitelyNotAProp="wrong"' : attributes;
            const start = line.indexOf(attribute);
            assert.ok(start >= 0);
            for (const diagnostic of actual) {
              assert.equal(diagnostic.file, "App.vue");
              assert.equal(diagnostic.line, 6);
              assert.ok(diagnostic.column >= start + 1);
              assert.ok(diagnostic.column <= start + attribute.length);
            }
          },
        );
      }
    }
    await captureCase(failures, "inherit-attrs-false", async () => {
      fs.writeFileSync(
        path.join(directory, "Forwarding.vue"),
        `<script setup lang="ts">
import Root from './Root.vue';
defineOptions({ inheritAttrs: false });
</script><template><Root /></template>`,
      );
      fs.writeFileSync(path.join(directory, "App.vue"), parent('<Forwarding bar="text" />'));
      const actual = await check(directory, ["--show-virtual-ts"], {
        name: "inherit-attrs-false",
        sources: ["App.vue", "Root.vue", "Forwarding.vue", "tsconfig.json"],
      });
      const expected = vueTscDiagnostics(directory);
      assert.deepEqual(
        expected.map((d) => d.code),
        [2353],
      );
      assert.deepEqual(actual.map(diagnosticIdentity), expected);
    });
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }

  if (failures.length) throw new AggregateError(failures, "listener input cases failed");
});

// Preserve the CLI's single authored finding after canonical deduplication.
// Each case retains its full process/source packet before any verdict, and
// later cases still run when an earlier assertion fails.
test("listener inference keeps owned props/emits ahead of inherited root names", async () => {
  const failures: Error[] = [];

  const directory = project({
    "Root.vue": `<script setup lang="ts">
defineProps<{ bar?: string }>();
defineEmits<{ click: [value: string]; foo: [value: string] }>();
</script><template><span /></template>`,
    "Forwarding.vue": `<script setup lang="ts">
import Root from './Root.vue';
defineProps<{ bar: number }>();
</script><template><Root /></template>`,
  });
  try {
    for (const [name, usage, code, count] of [
      ["own-number", ':bar="1" @foo="value => takesString(value)"', undefined, 0],
      ["wrong-own-number", 'bar="wrong" @foo="value => takesString(value)"', 2322, 1],
      ["inherited-click", ':bar="1" @click="value => takesString(value)"', undefined, 0],
      ["wrong-inherited-click", ':bar="1" @click="value => takesNumber(value)"', 2345, 1],
    ] as const) {
      await captureCase(failures, name, async () => {
        fs.writeFileSync(path.join(directory, "App.vue"), parent(`<Forwarding ${usage} />`));
        const actual = await check(directory, ["--show-virtual-ts"], {
          name,
          sources: ["App.vue", "Root.vue", "Forwarding.vue", "tsconfig.json"],
        });
        const oracle = vueTscDiagnostics(directory);
        assert.deepEqual(
          oracle.map((d) => d.code),
          code ? [code] : [],
        );
        assert.deepEqual(
          actual.map((d) => d.code),
          Array.from({ length: count }, () => code),
        );
        if (name === "wrong-inherited-click") {
          assert.deepEqual(actual.map(diagnosticIdentity), oracle);
          const line = fs.readFileSync(path.join(directory, "App.vue"), "utf8").split("\n")[5];
          assert.deepEqual(actual, [
            {
              file: "App.vue",
              line: 6,
              column: line.lastIndexOf("value") + 1,
              code: 2345,
              severity: "error",
              message:
                "Argument of type 'string' is not assignable to parameter of type 'number'. (source: <template><Forwarding :bar=\"1\" @click=\"value => takesNumber(value)\" /></template>)",
            },
          ]);
        }
        for (const diagnostic of actual) {
          assert.equal(diagnostic.file, "App.vue");
          assert.equal(diagnostic.line, 6);
        }
      });
    }
    fs.writeFileSync(
      path.join(directory, "Forwarding.vue"),
      `<script setup lang="ts">
import Root from './Root.vue';
defineProps<{ bar: number }>();
defineEmits<{ click: [value: number] }>();
</script><template><Root /></template>`,
    );
    for (const [name, handler, codes] of [
      ["own-click", "takesNumber", []],
      ["wrong-own-click", "takesString", [2345]],
    ] as const) {
      await captureCase(failures, name, async () => {
        fs.writeFileSync(
          path.join(directory, "App.vue"),
          parent(`<Forwarding :bar="1" @click="value => ${handler}(value)" />`),
        );
        const actual = await check(directory, ["--show-virtual-ts"], {
          name,
          sources: ["App.vue", "Root.vue", "Forwarding.vue", "tsconfig.json"],
        });
        const oracle = vueTscDiagnostics(directory);
        assert.deepEqual(
          oracle.map((d) => d.code),
          codes,
        );
        assert.deepEqual(actual.map(diagnosticIdentity), oracle);
      });
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }

  if (failures.length) throw new AggregateError(failures, "listener input cases failed");
});

test("listener inputs leave unknown-prop policy with its canonical checker", async () => {
  const failures: Error[] = [];

  for (const checkUnknownProps of [false, true]) {
    const directory = project(
      {
        "Root.vue": `<script setup lang="ts">
defineProps<{ bar?: string }>();
defineEmits<{ foo: [value: string] }>();
</script><template><span /></template>`,
        "Forwarding.vue": `<script setup lang="ts">
import Root from './Root.vue';
</script><template><Root /></template>`,
      },
      { checkUnknownProps },
    );
    try {
      for (const listener of ["", ' @foo="value => takesString(value)"']) {
        await captureCase(
          failures,
          `unknown-${checkUnknownProps}-${listener ? "listener" : "no-listener"}`,
          async () => {
            fs.writeFileSync(
              path.join(directory, "App.vue"),
              parent(
                `<Forwarding bar="text" definitelyNotAProp="allowed only when configured"${listener} />`,
              ),
            );
            const actual = await check(directory, ["--show-virtual-ts"], {
              name: `unknown-${checkUnknownProps}-${listener ? "listener" : "no-listener"}`,
              sources: ["App.vue", "Root.vue", "Forwarding.vue", "tsconfig.json"],
            });
            const oracle = vueTscDiagnostics(directory);
            assert.deepEqual(
              oracle.map((d) => d.code),
              checkUnknownProps ? [2353] : [],
            );
            assert.deepEqual(actual.map(diagnosticIdentity), oracle);
          },
        );
      }
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }

  if (failures.length) throw new AggregateError(failures, "listener input cases failed");
});

test("typed inherited indices still validate values and spread bags with unknown checks disabled", async () => {
  const failures: Error[] = [];

  for (const [name, props, own, extra] of [
    ["broad", "{ [key: string]: string }", "bar", "extra"],
    ["pattern", "Partial<Record<`data-${string}`, string>>", "data-owned", "data-extra"],
  ]) {
    const directory = project(
      {
        "Root.vue": `<script setup lang="ts">
defineProps<${props}>();
</script><template><span /></template>`,
        "Forwarding.vue": `<script setup lang="ts">
import Root from './Root.vue';
defineProps<{ '${own}'?: number }>();
defineEmits<{ close: [] }>();
</script><template><Root /></template>`,
      },
      { checkUnknownProps: false },
    );
    try {
      for (const spread of [false, true]) {
        for (const wrong of [false, true]) {
          await captureCase(
            failures,
            `index-${name}-${spread ? "spread" : "named"}-${wrong ? "wrong" : "valid"}`,
            async () => {
              const attributes = spread
                ? `v-bind="{ '${own}': 1, '${extra}': ${wrong ? "123" : "'text'"} }"`
                : `:${own}="1" :${extra}="${wrong ? "123" : "'text'"}"`;
              fs.writeFileSync(
                path.join(directory, "App.vue"),
                parent(`<Forwarding ${attributes} @close="() => {}" />`),
              );
              const captureName = `index-${name}-${spread ? "spread" : "named"}-${wrong ? "wrong" : "valid"}`;
              const actual = await check(directory, ["--show-virtual-ts"], {
                name: captureName,
                sources: ["App.vue", "Root.vue", "Forwarding.vue", "tsconfig.json"],
              });
              // vue-tsc accepts inherited indexed values; Canon additionally
              // validates their declared types, including values in spread bags.
              const oracle = vueTscDiagnostics(directory, captureName);
              assert.deepEqual(oracle, []);
              if (!wrong) assert.deepEqual(actual.map(diagnosticIdentity), oracle);
              assert.deepEqual(
                actual.map((d) => d.code),
                wrong ? [spread ? 2345 : 2322] : [],
              );
              for (const diagnostic of actual) {
                assert.equal(diagnostic.file, "App.vue");
                assert.equal(diagnostic.line, 6);
                const line = fs
                  .readFileSync(path.join(directory, "App.vue"), "utf8")
                  .split("\n")[5];
                const owner = spread ? "Forwarding" : `:${extra}="123"`;
                const start = line.indexOf(owner);
                assert.ok(start >= 0);
                assert.ok(diagnostic.column >= start + 1);
                assert.ok(diagnostic.column <= start + owner.length);
              }
            },
          );
        }
      }
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  }

  if (failures.length) throw new AggregateError(failures, "listener input cases failed");
});
