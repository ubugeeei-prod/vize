import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { resolveVizeLaunchCommand } from "./support/lsp/launch.ts";
import { root } from "./support/lsp/paths.ts";

const source = `<script setup lang="ts">
import Child from "./Child.vue";
const foo = 1;
const html = "<b>x</b>";
const list = [1, 2];
</script>
<template>
  <Child :foo="foo" />
  <div v-html="html"></div>
  <div v-for="item in list">{{ item }}</div>
</template>
`;

void test("source CLI top-level Vite+ lint config equals the complete canonical native diagnostic packet", (t) => {
  const [binary] = resolveVizeLaunchCommand(undefined, path.join(root, "target/ci/vize"), {
    required: true,
    repoRoot: root,
  });
  const directory = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-lint-config-")));
  const evidence = path.join(root, "target/differential/viteplus-lint-settings/cli.json");
  const rows = [];
  try {
    fs.mkdirSync(path.join(directory, "src"));
    fs.writeFileSync(path.join(directory, "package.json"), '{"type":"module"}');
    fs.symlinkSync(
      path.join(root, "node_modules"),
      path.join(directory, "node_modules"),
      "junction",
    );
    fs.writeFileSync(path.join(directory, "src/Comp.vue"), source);
    fs.writeFileSync(
      path.join(directory, "src/Child.vue"),
      '<script setup lang="ts">defineProps<{ foo: number }>();</script>\n<template><div /></template>\n',
    );
    const run = (config, extra = []) => {
      const selected = path.join(directory, "vite.config.ts");
      const configSource = `export default ${JSON.stringify(config)};\n`;
      fs.writeFileSync(selected, configSource);
      const start = performance.now();
      const args = ["lint", "src/Comp.vue", "--format", "json", ...extra];
      const result = spawnSync(binary, args, {
        cwd: directory,
        encoding: "utf8",
        timeout: 30_000,
        env: { ...process.env, NO_COLOR: "1" },
      });
      rows.push({
        configSource,
        args,
        status: result.status,
        signal: result.signal,
        error: result.error?.message ?? null,
        stdout: result.stdout,
        stderr: result.stderr,
        milliseconds: performance.now() - start,
        sourceAfter: fs.readFileSync(path.join(directory, "src/Comp.vue"), "utf8"),
      });
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(fs.readFileSync(path.join(directory, "src/Comp.vue"), "utf8"), source);
      return result;
    };
    const cases = [
      {
        preset: "opinionated",
        rules: {
          "vize/vue/prefer-props-shorthand": "warn",
          "vize/vue/no-v-html": "warn",
          "vize/vue/require-v-for-key": "error",
        },
        native: {
          preset: "opinionated",
          rules: {
            "vue/prefer-props-shorthand": "warn",
            "vue/no-v-html": "warn",
            "vue/require-v-for-key": "error",
          },
        },
      },
      {
        preset: "incremental",
        rules: {
          "vize/vue/prefer-props-shorthand": 1,
          "vize/vue/no-v-html": 0,
          "vize/vue/require-v-for-key": 2,
        },
        native: {
          preset: "incremental",
          rules: {
            "vue/prefer-props-shorthand": "warn",
            "vue/no-v-html": "off",
            "vue/require-v-for-key": "error",
          },
        },
      },
      {
        preset: "incremental",
        rules: { "vize/vue/html-quotes": ["warn", "single"] },
        native: {
          preset: "incremental",
          rules: { "vue/html-quotes": "warn" },
          ruleOptions: { "vue/html-quotes": "single" },
        },
      },
    ];
    for (const fixture of cases) {
      const canonical = run({ vue: { linter: fixture.native } });
      assert.ok(canonical.status === 0 || canonical.status === 1);
      const shared = run({
        lint: {
          plugins: ["vue"],
          jsPlugins: ["oxlint-plugin-vize"],
          settings: { vize: { preset: fixture.preset, helpLevel: "short" } },
          rules: { "no-console": "warn", ...fixture.rules },
        },
      });
      assert.equal(shared.status, canonical.status);
      assert.equal(
        shared.stdout,
        canonical.stdout,
        "all diagnostic identities, severities, messages and UTF-16 coordinates",
      );
      assert.equal(shared.stderr, canonical.stderr);
      assert.notDeepEqual(JSON.parse(shared.stdout), [], "a false-clean packet cannot qualify");
      if (fixture.preset === "opinionated") {
        const packet = JSON.parse(shared.stdout);
        assert.ok(
          packet
            .flatMap((file) => file.messages)
            .some((message) => message.ruleId === "vue/prefer-props-shorthand"),
        );
        assert.notEqual(
          run({}, ["--no-config"]).stdout,
          shared.stdout,
          "the original report distinguishes missing projection",
        );
      }
    }
    const common = {
      settings: { vize: { preset: "incremental" } },
      rules: { "vize/vue/prefer-props-shorthand": "warn", "vize/vue/no-v-html": "warn" },
    };
    for (const fixture of [
      {
        layer: true,
        native: {
          enabled: true,
          preset: "incremental",
          rules: { "vue/prefer-props-shorthand": "warn", "vue/no-v-html": "warn" },
        },
      },
      {
        layer: false,
        native: {
          enabled: false,
          preset: "incremental",
          rules: { "vue/prefer-props-shorthand": "warn", "vue/no-v-html": "warn" },
        },
      },
      {
        layer: false,
        canonical: { enabled: true, rules: { "vue/no-v-html": "off" } },
        native: {
          enabled: true,
          preset: "incremental",
          rules: { "vue/prefer-props-shorthand": "warn", "vue/no-v-html": "off" },
        },
      },
      {
        layer: { preset: "opinionated", rules: { "vue/no-v-html": "off" } },
        canonical: { preset: "essential" },
        native: {
          preset: "essential",
          rules: { "vue/prefer-props-shorthand": "warn", "vue/no-v-html": "off" },
        },
      },
    ]) {
      const canonical = run({ vue: { linter: fixture.native } });
      const shared = run({
        lint: { ...common, vue: fixture.layer },
        ...(fixture.canonical ? { vue: { linter: fixture.canonical } } : {}),
      });
      assert.equal(shared.status, canonical.status);
      assert.equal(
        shared.stdout,
        canonical.stdout,
        "complete boolean/object/canonical precedence packet",
      );
      assert.equal(shared.stderr, canonical.stderr);
    }
    const rejected = run({ lint: { rules: { "vize/vue/no-v-html": "warning" } } });
    assert.equal(rejected.status, 2);
    assert.equal(rejected.stdout, "");
    assert.match(rejected.stderr, /unsupported rule severity/);
  } finally {
    fs.mkdirSync(path.dirname(evidence), { recursive: true });
    fs.writeFileSync(evidence, JSON.stringify({ issue: 10619, rows }, null, 2) + "\n");
    fs.rmSync(directory, { recursive: true, force: true });
    t.diagnostic(`Complete source diagnostic packets: ${evidence}`);
  }
});
