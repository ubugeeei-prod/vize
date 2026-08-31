import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateFixtureTypePackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation.mjs";

/**
 * `references` walk the project graph; they do not merge programs. A leaf that
 * already has `paths` must not absorb another project's packages, a directory
 * reference must resolve `tsconfig.json`, and two referenced copies of one
 * name must not be guessed between (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-refs-")));
  const fixtureRoot = path.join(outer, "fixture");
  const store = path.join(fixtureRoot, "node_modules", ".pnpm");
  fs.mkdirSync(path.join(fixtureRoot, ".nuxt"), { recursive: true });
  for (const [name, id] of [
    ["vue-router", "vue-router@5.1.0"],
    ["defu", "defu@6.1.4"],
    ["vue-router", "vue-router@4.5.1"],
  ] as const) {
    const packageRoot = path.join(store, id, "node_modules", name);
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  }
  for (const name of ["vue-router", "defu"]) {
    const packageRoot = path.join(outer, "node_modules", name);
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  }
  return { outer, fixtureRoot };
}

function writeJson(filePath: string, value: unknown) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value)}\n`);
}

test("a config that already declares paths does not absorb packages from its references", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    const app = path.join(fixtureRoot, ".nuxt", "tsconfig.app.json");
    const node = path.join(fixtureRoot, ".nuxt", "tsconfig.node.json");
    writeJson(app, {
      compilerOptions: {
        paths: {
          "vue-router": ["../node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router"],
        },
      },
      references: [{ path: "./tsconfig.node.json" }],
    });
    writeJson(node, {
      compilerOptions: {
        paths: { defu: ["../node_modules/.pnpm/defu@6.1.4/node_modules/defu"] },
      },
    });
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, app), [
      { name: "vue-router", target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router" },
    ]);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "defu")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a relative directory reference isolates packages from that project's tsconfig.json", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeJson(path.join(fixtureRoot, ".nuxt", "tsconfig.json"), {
      compilerOptions: {
        paths: {
          "vue-router": ["../node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router"],
        },
      },
    });
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    writeJson(configPath, { files: [], references: [{ path: "./.nuxt" }] });
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), [
      { name: "vue-router", target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("conflicting referenced targets for one name are dropped rather than guessed", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeJson(path.join(fixtureRoot, ".nuxt", "tsconfig.app.json"), {
      compilerOptions: {
        paths: {
          "vue-router": ["../node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router"],
        },
      },
    });
    writeJson(path.join(fixtureRoot, ".nuxt", "tsconfig.node.json"), {
      compilerOptions: {
        paths: {
          "vue-router": ["../node_modules/.pnpm/vue-router@4.5.1/node_modules/vue-router"],
        },
      },
    });
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    writeJson(configPath, {
      files: [],
      references: [{ path: "./.nuxt/tsconfig.app.json" }, { path: "./.nuxt/tsconfig.node.json" }],
    });
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
