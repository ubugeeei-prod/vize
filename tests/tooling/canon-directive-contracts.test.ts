import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {
  check,
  workspace,
  compareIdentity,
  diagnosticIdentity,
} from "./support/upstream/vue-language-tools.ts";
import { assertVueTsc, vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";

function project(source: string): string {
  const directory = workspace("directive-hook-contracts-");
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        target: "ESNext",
        module: "ESNext",
        moduleResolution: "Bundler",
        noEmit: true,
        skipLibCheck: true,
      },
      include: ["*.vue"],
    }),
  );
  fs.writeFileSync(path.join(directory, "App.vue"), source);
  return directory;
}

test("directive hooks preserve generic tuple relationships and reject invalid values", async () => {
  const source = `<script setup lang="ts">
import type { ObjectDirective, FunctionDirective } from 'vue';
const vZero = () => {};
const vOne = (_element: HTMLElement) => {};
const vObject = {
  created<T extends number>(_el: HTMLElement, _binding: { value: [T, (a: T) => void] }) {},
  mounted<T extends number>(_el: HTMLElement, _binding: { value: [T, (a: T) => void] }, _vNode: any) {},
};
function vFunction<T extends number>(_el: HTMLElement, _binding: { value: [T, (a: T) => void] }) {}
declare const vDeclaredObject: ObjectDirective<HTMLElement, [1, (a: 1) => void]>;
declare const vDeclaredFunction: FunctionDirective<HTMLElement, [1, (a: 1) => void]>;
</script><template>
<div v-zero="123" /><div v-one="123" />
<div v-object="[1, (_a: 1) => {}]" />
<div v-function="[1, (_a: 1) => {}]" />
<div v-declared-object="[1, (_a: 1) => {}]" />
<div v-declared-function="[1, (_a: 1) => {}]" />
</template>`;
  const directory = project(source);
  try {
    assertVueTsc(directory);
    assert.deepEqual(await check(directory), []);
    assert.deepEqual(await check(directory, ["--declaration", "--declaration-dir", "types"]), []);
    // A passing upstream fixture alone cannot prove checking: calls to shorter
    // hooks must still reject values after synthetic vnode arguments are added.
    const invalid = source.replaceAll("[1, (_a: 1) => {}]", "'wrong'");
    fs.writeFileSync(path.join(directory, "App.vue"), invalid);
    const diagnostics = await check(directory);
    assert.equal(diagnostics.length, 4, JSON.stringify(diagnostics));
    for (const diagnostic of diagnostics) {
      assert.equal(diagnostic.code, 2322);
      const line = invalid.split("\n")[diagnostic.line - 1];
      assert.equal(line.slice(diagnostic.column - 1, diagnostic.column + 6), "'wrong'");
    }
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("directive callbacks and invalid values match vue-tsc diagnostic locations", async () => {
  const directory = project(`<script setup lang="ts">
import type { ObjectDirective, FunctionDirective } from 'vue';
declare const vObject: ObjectDirective<HTMLElement, (item: number) => void>;
declare const vFunction: FunctionDirective<HTMLElement, number>;
</script><template>
<div v-object="item => item.toFixed()" />
<div v-object="item => item.missing" />
<div v-function="'wrong'" />
</template>`);
  try {
    const expected = vueTscDiagnostics(directory).sort(compareIdentity);
    assert.equal(expected.length, 2);
    assert.deepEqual(
      (await check(directory)).map(diagnosticIdentity).sort(compareIdentity),
      expected,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
