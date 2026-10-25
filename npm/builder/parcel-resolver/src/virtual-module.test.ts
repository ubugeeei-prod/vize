import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { resolveVirtualModule } from "./virtual-module.ts";

const code = "// 😺 café\nexport default {};\n";
const digest = createHash("sha256").update(code).digest("hex");
const resolveFrom = "/app 😺 space/组件.vue";
const filePath = `${resolveFrom}.vize-${digest}.js`;
const dependency = () => ({
  specifier: `vize-generated-${digest}.js`,
  resolveFrom,
  meta: { vizeVirtualModule: { code, digest, filePath } },
});

void test("immutable Unicode component modules preserve exact bytes and relative import directory", () => {
  assert.deepEqual(resolveVirtualModule(dependency()), { filePath, code, sideEffects: true });
});

void test("ordinary dependencies fall through without a private payload", () => {
  assert.equal(resolveVirtualModule({ specifier: "vue", resolveFrom, meta: {} }), undefined);
  for (const entry of [null, [], "payload", false, 1]) {
    assert.throws(
      () =>
        resolveVirtualModule({
          specifier: "component.js",
          resolveFrom,
          meta: { vizeVirtualModule: entry },
        }),
      /Invalid immutable/,
    );
  }
});

void test("changed code, digest, specifier and module location cannot alias an existing request", () => {
  for (const change of [
    { code: `${code}export const added = 1;` },
    { digest: "a".repeat(64) },
    { digest: "../other" },
    { filePath: "/other.vue.js" },
  ]) {
    const input = dependency();
    Object.assign(input.meta.vizeVirtualModule, change);
    assert.throws(() => resolveVirtualModule(input), /Invalid immutable/);
  }
  const input = dependency();
  input.specifier = "other.js";
  assert.throws(() => resolveVirtualModule(input), /Invalid immutable/);
  input.specifier = dependency().specifier;
  input.resolveFrom = "relative.vue";
  assert.throws(() => resolveVirtualModule(input), /Invalid immutable/);
});
