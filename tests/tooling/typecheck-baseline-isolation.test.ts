import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { isolateFixtureTypePackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation.mjs";
import { materializeBaselineProject } from "../../legacy-tools/fixtures/typecheck-baseline-project.mjs";
import { typecheckDependencySkip } from "./support/typecheck-dependency.ts";

/**
 * Run 31979524200: close the one hole a fixture's own `compilerOptions.paths`
 * cannot.
 *
 * TypeScript resolves `/// <reference types="X" />` by walking `node_modules`
 * upward from the containing file and never consults `paths`. A fixture sits at
 * `tests/_fixtures/_git/<id>`, so that walk reaches Vize's own `node_modules`,
 * and on run 31979524200 elk's `.nuxt/nuxt.d.ts` asking for `vue-router` was
 * answered from there — pulling Vize's `vue@3.6.0-beta.10` in beside elk's own
 * `vue@3.5.30` and splitting every `declare module 'vue'` the fixture owns.
 *
 * The tree below is that shape: a package the fixture depends on but its package
 * manager did not hoist, a copy of the same name one directory above the
 * fixture, and a config that already maps the name to the fixture's own copy.
 */

function scaffold() {
  // Realpath, so vue-tsc's cwd-relative diagnostic paths are not prefixed with
  // the `/var` to `/private/var` climb macOS would otherwise put in front.
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-")));
  const fixtureRoot = path.join(outer, "fixture");
  const store = path.join(fixtureRoot, "node_modules", ".pnpm");
  fs.mkdirSync(path.join(fixtureRoot, ".nuxt"), { recursive: true });
  // The fixture's own copies, kept out of the top level exactly as pnpm does.
  for (const [name, id] of [
    ["vue-router", "vue-router@5.1.0"],
    ["@vue/runtime-core", "@vue+runtime-core@3.5.30"],
  ] as const) {
    const packageRoot = path.join(store, id, "node_modules", name);
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  }
  // Vize's own install, one directory above the fixture.
  for (const name of ["vue-router", "@vue/runtime-core", "unrelated"]) {
    const packageRoot = path.join(outer, "node_modules", name);
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  }
  return { outer, fixtureRoot, store };
}

function writeConfig(fixtureRoot: string, paths: Record<string, string[]>) {
  const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.app.json");
  fs.writeFileSync(configPath, `${JSON.stringify({ compilerOptions: { paths } }, null, 2)}\n`);
  return configPath;
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const vueTsc = path.join(
  process.env.VIZE_TEST_WORKSPACE_NODE_MODULES ?? path.join(repoRoot, "tests/node_modules"),
  ".bin/vue-tsc",
);
const vueTscOptions = {
  skip: typecheckDependencySkip(
    fs.existsSync(vueTsc) ? vueTsc : undefined,
    "vue-tsc for the baseline isolation gate",
    "vue-tsc binary unavailable",
  ),
};

function writePackage(packageRoot: string, marker: string) {
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(
    path.join(packageRoot, "package.json"),
    `{"name":"vue-router","types":"index.d.ts"}\n`,
  );
  fs.writeFileSync(
    path.join(packageRoot, "index.d.ts"),
    `declare global { const WHICH_COPY: "${marker}" }\nexport {}\n`,
  );
}

function runVueTsc(project: string, cwd: string) {
  return spawnSync(vueTsc, ["--noEmit", "--pretty", "false", "-p", project], {
    cwd,
    encoding: "utf8",
  });
}

/** vue-tsc reports paths relative to its cwd, so only the fixture prefix varies. */
function diagnostics(stdout: string, fixtureRoot: string) {
  return stdout
    .split("\n")
    .filter((line) => /error TS\d+/u.test(line))
    .map((line) => line.replaceAll(`${fixtureRoot}/`, "").trim());
}

const declaredPaths = {
  "vue-router": ["../node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router"],
  "@vue/runtime-core": [
    "../node_modules/.pnpm/@vue+runtime-core@3.5.30/node_modules/@vue/runtime-core",
  ],
  "~": ["../app"],
  "~/*": ["../app/*"],
  "#imports": ["./imports"],
};

test("a declared package an ancestor could answer is linked from the fixture's own copy", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, declaredPaths);
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), [
      {
        name: "@vue/runtime-core",
        target: "node_modules/.pnpm/@vue+runtime-core@3.5.30/node_modules/@vue/runtime-core",
      },
      { name: "vue-router", target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router" },
    ]);
    // The link has to be the resolver's answer, not just a file that exists:
    // reading through it must land inside the fixture rather than above it.
    for (const name of ["vue-router", "@vue/runtime-core"]) {
      const linked = fs.realpathSync(path.join(fixtureRoot, "node_modules", name));
      assert.equal(linked.startsWith(fs.realpathSync(fixtureRoot) + path.sep), true, name);
    }
    // Non-package keys are not names, so nothing is invented for them.
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "~")), false);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "#imports")), false);
    // A second run has nothing left to do, so a re-prepared shard is stable.
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), []);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a name no ancestor provides is left alone, because nothing can escape to", () => {
  const { outer, fixtureRoot, store } = scaffold();
  try {
    const packageRoot = path.join(store, "defu@6.1.4", "node_modules", "defu");
    fs.mkdirSync(packageRoot, { recursive: true });
    fs.writeFileSync(path.join(packageRoot, "package.json"), '{"name":"defu"}\n');
    const configPath = writeConfig(fixtureRoot, {
      defu: ["../node_modules/.pnpm/defu@6.1.4/node_modules/defu"],
    });
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "defu")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a name the fixture already hoists is left exactly as its package manager wrote it", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    const hoisted = path.join(fixtureRoot, "node_modules", "vue-router");
    fs.mkdirSync(hoisted, { recursive: true });
    fs.writeFileSync(path.join(hoisted, "package.json"), '{"name":"vue-router","version":"4"}\n');
    const configPath = writeConfig(fixtureRoot, declaredPaths);
    assert.deepEqual(
      isolateFixtureTypePackages(fixtureRoot, configPath).map((entry) => entry.name),
      ["@vue/runtime-core"],
    );
    assert.equal(fs.lstatSync(hoisted).isSymbolicLink(), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a declared target outside the fixture is never linked in", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    // Nuxt writes this when it resolved the package above the fixture itself.
    // Materializing it would import the contamination rather than close it.
    const configPath = writeConfig(fixtureRoot, {
      "vue-router": ["../../node_modules/vue-router"],
    });
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a declared target that is not a package directory is never linked in", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    // `paths` legitimately points at bare module stems and source folders; only
    // a real package directory can answer a type reference directive.
    fs.mkdirSync(path.join(fixtureRoot, "mocks", "vue-router"), { recursive: true });
    const configPath = writeConfig(fixtureRoot, { "vue-router": ["../mocks/vue-router"] });
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

/**
 * The oracle for all of the above: a real vue-tsc, on the real resolver, showing
 * that the escape happens and that the link closes it. `paths` already maps the
 * name to the fixture's copy in both runs — the only thing that changes is
 * whether the fixture answers before the walk leaves it.
 */
test(
  "real vue-tsc leaves the fixture for a type reference until the link exists",
  vueTscOptions,
  () => {
    const { outer, fixtureRoot } = scaffold();
    try {
      const reportDir = path.join(outer, "report");
      fs.mkdirSync(reportDir);
      // Same name, different declaration, so the diagnostic says which one won.
      writePackage(path.join(outer, "node_modules", "vue-router"), "foreign");
      writePackage(
        path.join(fixtureRoot, "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router"),
        "fixture",
      );
      fs.writeFileSync(
        path.join(fixtureRoot, ".nuxt", "nuxt.d.ts"),
        '/// <reference types="vue-router" />\nexport {}\n',
      );
      const configPath = writeConfig(fixtureRoot, {
        "vue-router": ["../node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router"],
      });
      fs.mkdirSync(path.join(fixtureRoot, "src"));
      fs.writeFileSync(
        path.join(fixtureRoot, "src/main.ts"),
        'export const which: "fixture" = WHICH_COPY\n',
      );
      fs.writeFileSync(
        configPath,
        `${JSON.stringify({
          compilerOptions: {
            strict: true,
            noEmit: true,
            module: "preserve",
            moduleResolution: "Bundler",
            target: "ESNext",
            paths: {
              "vue-router": ["../node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router"],
            },
          },
          include: ["./nuxt.d.ts", "../src/**/*"],
        })}\n`,
      );
      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        {
          id: "fixture",
          tsconfig: "tsconfig.json",
          typecheckPerformance: { baseline: { tsconfig: ".nuxt/tsconfig.app.json" } },
        },
        { fileCount: 1, files: [{ file: "src/main.ts" }] },
      );

      const escaped = runVueTsc(project.path, fixtureRoot);
      assert.deepEqual(diagnostics(escaped.stdout, fixtureRoot), [
        `src/main.ts(1,14): error TS2322: Type '"foreign"' is not assignable to type '"fixture"'.`,
      ]);

      assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), [
        {
          name: "vue-router",
          target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router",
        },
      ]);
      const isolated = runVueTsc(project.path, fixtureRoot);
      assert.deepEqual(diagnostics(isolated.stdout, fixtureRoot), []);
      assert.equal(isolated.status, 0, isolated.stderr);
    } finally {
      fs.rmSync(outer, { recursive: true, force: true });
    }
  },
);

test("a JSONC config that extends another still isolates the parent's packages", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeConfig(fixtureRoot, declaredPaths);
    const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.check.json");
    // reka-ui's check config is JSONC with `extends` and no `paths` of its own.
    fs.writeFileSync(configPath, `// check-only\n{ "extends": "./tsconfig.app.json", }\n`);
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), [
      {
        name: "@vue/runtime-core",
        target: "node_modules/.pnpm/@vue+runtime-core@3.5.30/node_modules/@vue/runtime-core",
      },
      { name: "vue-router", target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a solution-style config still isolates packages declared in relative referenced projects", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeConfig(fixtureRoot, declaredPaths);
    // elk's root tsconfig is a solution: `files: []` and `references`, with the
    // real `paths` on `.nuxt/tsconfig.app.json`. Package-name references are
    // not relative and must not be invented as configs.
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({
        files: [],
        references: [{ path: "@vue/tsconfig" }, { path: "./.nuxt/tsconfig.app.json" }],
      })}\n`,
    );
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, configPath), [
      {
        name: "@vue/runtime-core",
        target: "node_modules/.pnpm/@vue+runtime-core@3.5.30/node_modules/@vue/runtime-core",
      },
      { name: "vue-router", target: "node_modules/.pnpm/vue-router@5.1.0/node_modules/vue-router" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a config that declares nothing, or cannot be read, links nothing", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    const missing = path.join(fixtureRoot, "tsconfig.json");
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, missing), []);
    fs.writeFileSync(missing, "{ /* JSONC with no paths still declares nothing */ }\n");
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, missing), []);
    fs.writeFileSync(missing, '{"compilerOptions":{}}\n');
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, missing), []);
    fs.writeFileSync(missing, "{ this is not json\n");
    assert.deepEqual(isolateFixtureTypePackages(fixtureRoot, missing), []);
    assert.equal(fs.existsSync(path.join(fixtureRoot, "node_modules", "vue-router")), false);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
