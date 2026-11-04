import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const root = new URL(
  "../../../../tests/_fixtures/differential/compiler/unicode-script-line-comments-11798/",
  import.meta.url,
);
const nativeEntry = new URL("../../../native/index.js", import.meta.url);
const fromNative = createRequire(nativeEntry);
const native = fromNative(fileURLToPath(nativeEntry)) as typeof import("@vizejs/native");

test("#11798 real NAPI preserves all 36 scripts and complete module shapes", () => {
  const bindings = Object.values(fromNative.cache).filter(
    (module) =>
      module?.filename.endsWith(".node") && module.exports.compileSfc === native.compileSfc,
  );
  assert.equal(bindings.length, 1, "one actual source NAPI image");
  const binding = realpathSync(bindings[0]!.filename);
  assert.equal(dirname(binding), realpathSync(new URL("../../../native/", import.meta.url)));
  const checkout = execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: fileURLToPath(new URL("../../../../", import.meta.url)),
    encoding: "utf8",
  }).trim();
  if (process.env.GITHUB_SHA) assert.equal(checkout, process.env.GITHUB_SHA);
  const custody = {
    checkout,
    binding,
    binarySha256: createHash("sha256").update(readFileSync(binding)).digest("hex"),
    token: native.getNativeBuildToken(),
  };
  assert.match(custody.token!, /^[0-9a-f]{32}$/);
  const registration = JSON.parse(readFileSync(new URL("corpus.json", root), "utf8"));
  const bytes = readFileSync(new URL(registration.path, root));
  assert.equal(createHash("sha256").update(bytes).digest("hex"), registration.sha256);
  const corpus = JSON.parse(bytes.toString());
  assert.equal(corpus.cases.length, 36);
  const observations = [];
  try {
    for (const row of corpus.cases) {
      const result = native.compileSfc(row.source, {
        filename: "Case.vue",
        ssr: row.ssr,
        sourceMap: false,
      });
      observations.push({ id: row.id, source: row.source, result });
      assert.deepEqual(
        {
          code: result.code,
          errors: result.errors,
          warnings: result.warnings,
          moduleShape: result.moduleShape,
        },
        { code: row.code, errors: [], warnings: [], moduleShape: row.moduleShape },
        row.id,
      );
    }
  } finally {
    console.log("11798_NAPI_RAW", JSON.stringify({ custody, observations }));
  }
});
