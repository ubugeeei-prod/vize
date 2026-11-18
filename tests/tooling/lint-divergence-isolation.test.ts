import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  ancestorWithPackages,
  isolationRoot,
  relocatePaths,
  withIsolatedFixture,
} from "../../tools/support/fixtures/lint_divergence/isolation.ts";

test("a fixture mirror resolves packages only from the fixture's own install", () => {
  const repository = fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-repo-"));
  const fixture = path.join(repository, "tests/_fixtures/_git/app");
  fs.mkdirSync(path.join(repository, "tests/node_modules/reka-ui"), { recursive: true });
  fs.mkdirSync(path.join(fixture, "src"), { recursive: true });
  fs.mkdirSync(path.join(fixture, "node_modules/vue"), { recursive: true });
  fs.writeFileSync(path.join(fixture, "src/App.vue"), "<template><div /></template>\n");
  fs.writeFileSync(path.join(fixture, "tsconfig.json"), "{}\n");
  fs.symlinkSync("App.vue", path.join(fixture, "src/Alias.vue"));
  assert.equal(ancestorWithPackages(path.join(fixture, "src")), fixture);
  assert.equal(ancestorWithPackages(path.dirname(fixture)), path.join(repository, "tests"));

  const root = isolationRoot();
  let seen = "";
  withIsolatedFixture(fixture, root, "app", (mirror) => {
    seen = mirror;
    assert.equal(ancestorWithPackages(path.dirname(mirror)), null);
    assert.equal(
      fs.readFileSync(path.join(mirror, "src/App.vue"), "utf8"),
      "<template><div /></template>\n",
    );
    assert.equal(fs.readlinkSync(path.join(mirror, "src/Alias.vue")), "App.vue");
    assert.equal(
      fs.realpathSync(path.join(mirror, "node_modules")),
      fs.realpathSync(path.join(fixture, "node_modules")),
    );
    assert.equal(fs.existsSync(path.join(mirror, "tsconfig.json")), true);
    assert.deepEqual(
      relocatePaths(
        { filePath: path.join(mirror, "src/App.vue"), other: [mirror, `${mirror}x`] },
        mirror,
        fixture,
      ),
      { filePath: path.join(fixture, "src/App.vue"), other: [fixture, `${mirror}x`] },
    );
  });
  assert.equal(fs.existsSync(seen), false);
  assert.equal(fs.existsSync(path.join(fixture, "src/App.vue")), true);
  assert.throws(() => isolationRoot(fixture), /resolves packages from/u);
  fs.rmSync(repository, { recursive: true, force: true });
});
