import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { captureCurrentSdkDeclarations } from "../../tools/support/compat/content-mapper/current-sdk-declarations.ts";

const source = fileURLToPath(new URL("../../npm/cli", import.meta.url));

void test("the current manifest and actual declaration graph cover mapper and checker consumers", () => {
  const closure = captureCurrentSdkDeclarations(source);
  assert.equal(closure.files.length, 12);
  assert.equal(closure.graph.files.length, 12);
  assert.equal(closure.graph.edges.length, 18);
  assert.equal(closure.mapperGraph.files.length, 11);
  assert.equal(closure.mapperGraph.edges.length, 17);
  assert.ok(closure.files.includes("virtual-ts-emits.d.ts"));
  assert.ok(closure.files.includes("virtual-ts-checker.d.ts"));
});

void test("current consumers reject missing, unshipped, disconnected or publicly exposed private declarations", () => {
  const parent = fs.mkdtempSync(path.join(os.tmpdir(), "vize-current-sdk-closure-"));
  const closure = captureCurrentSdkDeclarations(source);
  try {
    for (const control of [
      "missing-leaf",
      "unshipped-leaf",
      "public-leaf",
      "public-js-leaf",
      "public-extensionless-leaf",
      "public-pattern",
      "missing-entry-edge",
      "missing-base-edge",
      "missing-relative-target",
      "unshipped-public-entry",
      "changed-mode",
    ] as const) {
      const directory = path.join(parent, control);
      fs.mkdirSync(directory);
      for (const file of ["package.json", ...closure.files]) {
        const target = path.join(directory, file);
        fs.mkdirSync(path.dirname(target), { recursive: true });
        fs.copyFileSync(path.join(source, file), target);
      }
      const manifestPath = path.join(directory, "package.json");
      const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as {
        files: string[];
        exports: Record<string, unknown>;
      };
      const leaf = path.join(directory, "virtual-ts-emits.d.ts");
      if (control === "missing-leaf") fs.unlinkSync(leaf);
      if (control === "unshipped-leaf")
        manifest.files = manifest.files.filter((file) => file !== "virtual-ts-emits.d.ts");
      if (control === "public-leaf")
        manifest.exports["./private-alias"] = { types: "./virtual-ts-emits.d.ts" };
      if (control === "public-js-leaf")
        manifest.exports["./private-alias"] = { types: "./virtual-ts-emits.js" };
      if (control === "public-extensionless-leaf")
        manifest.exports["./private-alias"] = { types: "./virtual-ts-emits" };
      if (control === "public-pattern") manifest.exports["./*"] = { types: "./*.d.ts" };
      if (control === "missing-entry-edge" || control === "changed-mode") {
        const entry = path.join(directory, "virtual-ts.d.cts");
        const text = fs.readFileSync(entry, "utf8");
        const changed =
          control === "missing-entry-edge"
            ? text.replace(/^import type \* as __vizeEmitTypes.*\n/gmu, "")
            : text.replace(
                'from "./virtual-ts-emits.js" with { "resolution-mode": "import" }',
                'from "./virtual-ts-emits.js" with { "resolution-mode": "require" }',
              );
        assert.notEqual(changed, text);
        fs.writeFileSync(entry, changed);
      }
      if (control === "missing-base-edge") {
        const text = fs.readFileSync(leaf, "utf8");
        const changed = text.replace(
          /import type \{[^}]*\} from "\.\/virtual-ts\.js" with \{[^}]*\};\n/u,
          "",
        );
        assert.notEqual(changed, text);
        fs.writeFileSync(leaf, changed);
      }
      if (control === "missing-relative-target")
        fs.appendFileSync(leaf, '\nimport type {} from "./missing.js";\n');
      if (control === "unshipped-public-entry")
        manifest.files = manifest.files.filter((file) => file !== "virtual-ts-checker.d.ts");
      fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2) + "\n");
      assert.throws(() => captureCurrentSdkDeclarations(directory), control);
    }
  } finally {
    fs.rmSync(parent, { recursive: true, force: true });
  }
});
