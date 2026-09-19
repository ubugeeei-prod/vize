import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { check, workspace } from "./support/upstream/vue-language-tools.ts";
import { assertVueTsc } from "./support/vue-tsc-oracle.ts";

const assertion = `type Equal<X, Y> = (<T>() => T extends X ? 1 : 2) extends (<T>() => T extends Y ? 1 : 2) ? X : never;
declare function exact<X, Y>(actual: X & Equal<X, Y>, expected: Y & Equal<X, Y>): void;`;

function project(prefix: string, options: Record<string, unknown> = {}): string {
  const directory = workspace(prefix);
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        module: "esnext",
        moduleResolution: "bundler",
        skipLibCheck: true,
      },
      vueCompilerOptions: options,
      include: ["*.vue", "*.ts"],
    }),
  );
  return directory;
}

function source(strict: boolean, comment = ""): string {
  return `${comment}<script setup lang="ts">
import { useCssModule } from 'vue';
${assertion}
const styles = useCssModule();
exact(styles, {} as ${strict ? "" : "Record<string, string> & "}{ root: string });
styles.root = 'writable';
${strict ? "// @ts-expect-error\n" : ""}styles.missing;
const named = useCssModule('tokens');
exact(named, {} as ${strict ? "" : "Record<string, string> & "}{ active: string });
// @ts-expect-error
useCssModule('missing-module');
</script>
<template><div :class="$style.root" />${strict ? "<!-- @vue-expect-error -->" : ""}<div :class="$style.missing" /></template>
<style module>.root {}</style>
<style module="tokens">.active {}</style>`;
}

test("CSS module strictness follows project inheritance and authored top-level options", async () => {
  const directory = project("css-options-contract-");
  try {
    for (const [projectStrict, fileComment, effective] of [
      [false, "", false],
      [true, "", true],
      [false, "<!-- @strictCssModules true -->", true],
      [true, "<!-- @strictCssModules false -->", false],
      [false, "<!-- @strictTemplates true -->", false],
    ] as const) {
      const config = JSON.parse(fs.readFileSync(path.join(directory, "tsconfig.json"), "utf8"));
      config.vueCompilerOptions = {};
      config.extends = "./base.json";
      fs.writeFileSync(
        path.join(directory, "base.json"),
        JSON.stringify({ vueCompilerOptions: { strictCssModules: projectStrict } }),
      );
      fs.writeFileSync(path.join(directory, "tsconfig.json"), JSON.stringify(config));
      fs.writeFileSync(path.join(directory, "App.vue"), source(effective, fileComment));
      assertVueTsc(directory);
      assert.deepEqual(await check(directory), [], `${projectStrict}/${fileComment}`);
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("CSS helper specialization respects aliases, nested lexical shadows and expression positions", async () => {
  const directory = project("css-scope-contract-", { strictCssModules: true });
  try {
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      `<script setup lang="ts">
import { useCssModule as css } from 'vue';
import * as Vue from 'vue';
${assertion}
exact(Vue.useCssModule().root, '' as string);
exact(Vue.ref(1).value, 0 as number);
// @ts-expect-error
Vue.useCssModule().absent;
exact(css().root, '' as string);
const nested = () => css('tokens').active;
exact(nested(), '' as string);
function shadow(css: () => { authored: 1 }) { return css().authored; }
exact(shadow(() => ({ authored: 1 })), 1 as const);
// @ts-expect-error
css().missing;
// @ts-expect-error
css('missing-module');
const newline = css(
  'tokens'
);
exact(newline, {} as { active: string });
</script><style module>.root {}</style><style module="tokens">.active {}</style>`,
    );
    assert.deepEqual(await check(directory), []);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("CSS module absence stays a type error and arbitrary module names remain valid", async () => {
  const directory = project("css-names-contract-");
  try {
    fs.writeFileSync(
      path.join(directory, "Empty.vue"),
      `<script setup lang="ts">
import { useCssModule } from 'vue';
// @ts-expect-error
useCssModule();
// @ts-expect-error
useCssModule('none');
</script>`,
    );
    assertVueTsc(directory);
    // Vue accepts arbitrary CSS-module names; language-tools 3.3.11 still emits
    // an invalid unquoted type key for hyphenated names. Keep this regression
    // independent of that version-specific oracle limitation.
    fs.writeFileSync(
      path.join(directory, "App.vue"),
      `<script setup lang="ts">
import { useCssModule } from 'vue';
${assertion}
const styles = useCssModule('tokens-dark');
exact(styles, {} as Record<string, string> & { root: string });
</script><template><div :class="styles.root" /></template><style module="tokens-dark">.root {}</style>`,
    );
    assert.deepEqual(await check(directory), []);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
