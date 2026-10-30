import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { makeSourceProject, sha } from "./support/content-mapper-instance-globals-10617.ts";

test("10617 source adapter preserves all seven current SDK declarations and public type exports", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-instance-globals-helper-image-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const repoRoot = fileURLToPath(new URL("../../", import.meta.url));
  const published = path.join(root, "published");
  fs.mkdirSync(path.join(published, "node_modules"), { recursive: true });
  const binary = path.join(root, "source-cli-identity");
  fs.writeFileSync(binary, "source CLI identity control; not an executable consumer\n");
  const project = path.join(root, "source");
  const receipt = makeSourceProject(project, published, repoRoot, binary, new Map());
  const adapter = path.join(project, "node_modules/vize");
  const helpers = [
    "content-mapper-helpers.d.ts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.cts",
    "generated/content-mapper-types.d.ts",
    "virtual-ts.d.ts",
    "virtual-ts.d.mts",
    "virtual-ts.d.cts",
  ];
  for (const name of helpers) {
    assert.deepEqual(
      fs.readFileSync(path.join(adapter, name)),
      fs.readFileSync(path.join(repoRoot, "npm/cli", name)),
      name,
    );
  }
  const manifest = JSON.parse(fs.readFileSync(path.join(repoRoot, "npm/cli/package.json"), "utf8"));
  manifest.private = true;
  manifest.typescript.contentMapper.exec = [binary, "content-mapper"];
  const manifestBytes = `${JSON.stringify(manifest, null, 2)}\n`;
  assert.equal(fs.readFileSync(path.join(adapter, "package.json"), "utf8"), manifestBytes);
  assert.deepEqual(
    manifest.files.filter((name: string) => /\.d\.(?:ts|mts|cts)$/u.test(name)).sort(),
    [...helpers].sort(),
    "every declaration advertised by the actual package is materialized",
  );
  const exports = JSON.parse(fs.readFileSync(path.join(adapter, "package.json"), "utf8")).exports;
  for (const [subpath, stem] of [
    ["./virtual-ts", "virtual-ts"],
    ["./content-mapper-helpers", "content-mapper-helpers"],
  ]) {
    const selected = {
      import: `./${stem}.d.mts`,
      require: `./${stem}.d.cts`,
      types: `./${stem}.d.ts`,
    };
    assert.deepEqual(exports[subpath], {
      import: { types: selected.import },
      require: { types: selected.require },
      types: selected.types,
    });
    for (const condition of ["import", "require", "types"] as const) {
      const target: string =
        condition === "types" ? exports[subpath].types : exports[subpath][condition].types;
      assert.equal(target, selected[condition]);
      assert.ok(fs.lstatSync(path.join(adapter, target)).isFile(), `${subpath}/${condition}`);
      assert.deepEqual(
        fs.readFileSync(path.join(adapter, target)),
        fs.readFileSync(path.join(repoRoot, "npm/cli", target)),
        `${subpath}/${condition}`,
      );
    }
  }
  for (const name of [
    "content-mapper-helpers.d.ts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.cts",
  ]) {
    assert.match(
      fs.readFileSync(path.join(adapter, name), "utf8"),
      /export type \* from "\.\/generated\/content-mapper-types\.js"/u,
      `${name} retains its real transitive declaration dependency`,
    );
    assert.ok(fs.lstatSync(path.join(adapter, "generated/content-mapper-types.d.ts")).isFile());
  }
  assert.deepEqual(receipt, {
    role: "source protocol adapter, not public-package acceptance",
    manifestSha256: sha(manifestBytes),
    binarySha256: sha(fs.readFileSync(binary)),
    helperFiles: helpers.map((name) => ({
      name,
      sha256: sha(fs.readFileSync(path.join(repoRoot, "npm/cli", name))),
    })),
  });
  assert.deepEqual(fs.readdirSync(adapter).sort(), [
    "content-mapper-helpers.d.cts",
    "content-mapper-helpers.d.mts",
    "content-mapper-helpers.d.ts",
    "generated",
    "package.json",
    "virtual-ts.d.cts",
    "virtual-ts.d.mts",
    "virtual-ts.d.ts",
  ]);
  assert.deepEqual(fs.readdirSync(path.join(adapter, "generated")), ["content-mapper-types.d.ts"]);
});
