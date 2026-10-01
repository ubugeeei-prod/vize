import assert from "node:assert/strict";
import { test } from "node:test";

import { metadata, readRepoFile } from "./support/davinci-stage-dependencies.ts";
import { scanConsumerMigrationSurfaces } from "../../tools/support/compat/davinci/lib/consumer-migration-scan.mjs";

test("stage catalogue names every actual level package and excludes host facades", () => {
  const source = readRepoFile("davinci/vize_l0/src/stage.rs");
  const levels = new Map(
    [...source.matchAll(/pub const (L\d): LayerCrate = LayerCrate \{([^}]+)\};/gu)].map(
      ([, symbol, body]) => [symbol, body],
    ),
  );
  const listed = source.match(/pub const LAYERS[^=]+ = &\[([^\]]+)\];/u)?.[1];
  assert.ok(listed);
  assert.deepEqual(
    listed
      .split(",")
      .map((name) => name.trim())
      .sort(),
    [...levels.keys()].sort(),
  );
  const packages = metadata.packages.filter((pkg) => /^vize_l\d$/u.test(pkg.name));
  assert.equal(levels.size, packages.length);
  for (const pkg of packages) {
    const symbol = pkg.name.slice("vize_".length).toUpperCase();
    const body = levels.get(symbol);
    assert.ok(body, `${pkg.name} is missing from the exported catalogue`);
    for (const field of ["crate_alias", "package"]) {
      assert.equal(body.match(new RegExp(`${field}: "([^"]+)"`, "u"))?.[1], pkg.name);
    }
  }
});

test("Carton consumer mentions stay visible without receiving native level credit", () => {
  const scan = scanConsumerMigrationSurfaces();
  const sites = scan.consumers
    .flatMap((consumer) => consumer.sites)
    .filter((site) => site.matchedName === "vize_carton");
  assert.ok(sites.length > 0, "real host callers must remain observable");
  for (const site of sites) {
    assert.equal(site.surfaceId, "carton");
    assert.equal(site.group, "old");
    assert.equal(site.nameKind, "legacy");
  }
});
