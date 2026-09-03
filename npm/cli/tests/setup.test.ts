import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { test } from "vite-plus/test";

import { setupProject, type SetupCommand } from "../src/setup.ts";

function temporaryProject(name: string): string {
  return fs.mkdtempSync(path.join(os.tmpdir(), `vize-setup-${name}-`));
}

function write(root: string, filename: string, source: string): void {
  fs.writeFileSync(path.join(root, filename), source);
}

function read(root: string, filename: string): string {
  return fs.readFileSync(path.join(root, filename), "utf8");
}

function packageJson(root: string): Record<string, unknown> {
  return JSON.parse(read(root, "package.json")) as Record<string, unknown>;
}

test("configures a canonical Vite+ project and is byte-idempotent", () => {
  const root = temporaryProject("canonical");
  const commands: SetupCommand[] = [];
  try {
    write(
      root,
      "package.json",
      `${JSON.stringify(
        {
          name: "fixture",
          private: true,
          type: "module",
          scripts: { dev: "vp dev" },
          devDependencies: {
            "@vitejs/plugin-vue": "^6.0.0",
            "vite-plus": "^0.1.0",
          },
        },
        null,
        2,
      )}\n`,
    );
    write(
      root,
      "vite.config.ts",
      `import { defineConfig } from "vite-plus";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
});
`,
    );

    const result = setupProject({
      root,
      runCommand(command) {
        commands.push(command);
        const manifest = packageJson(root);
        const devDependencies = (manifest.devDependencies ?? {}) as Record<string, string>;
        if (command.args[0] === "add") {
          for (const dependency of command.args.slice(2)) {
            devDependencies[dependency] = "^0.300.0";
          }
        } else {
          delete devDependencies["@vitejs/plugin-vue"];
        }
        manifest.devDependencies = devDependencies;
        write(root, "package.json", `${JSON.stringify(manifest, null, 2)}\n`);
      },
    });

    assert.deepEqual(commands, [
      {
        command: "vp",
        args: [
          "add",
          "-D",
          "vize",
          "@vizejs/vite-plugin",
          "@vizejs/vite-plugin-musea",
          "oxlint",
          "oxlint-plugin-vize",
        ],
        cwd: root,
      },
      {
        command: "vp",
        args: ["remove", "@vitejs/plugin-vue"],
        cwd: root,
      },
    ]);
    assert.deepEqual(result.createdFiles, ["vize.config.ts"]);
    assert.equal(result.migratedViteConfig, "vite.config.ts");
    assert.equal(result.enabledVitePlusLint, true);
    assert.match(read(root, "vite.config.ts"), /from "@vizejs\/vite-plugin"/u);
    assert.match(read(root, "vite.config.ts"), /plugins: \[vue\(\)\]/u);
    assert.match(read(root, "vite.config.ts"), /createVizeLintConfig/u);
    assert.match(read(root, "vite.config.ts"), /preset: "happy-path"/u);
    assert.match(read(root, "vize.config.ts"), /defineConfig/u);
    assert.equal(fs.existsSync(path.join(root, "oxlint.config.ts")), false);

    const scripts = packageJson(root).scripts as Record<string, string>;
    assert.equal(scripts.dev, "vp dev");
    assert.equal(scripts["vize:fmt"], "vize fmt --check src");
    assert.equal(scripts["vize:fmt:fix"], "vize fmt --write src");
    assert.equal(scripts["vize:lint"], "vize lint --preset happy-path --max-warnings 0 src");
    assert.equal(scripts["vize:check"], "vize check");
    assert.equal(scripts["vize:build"], "vize build src");
    assert.equal(scripts["vize:musea"], "vize musea");
    assert.equal(scripts["vize:ready"], "vize ready src");

    const firstSources = new Map(
      ["package.json", "vite.config.ts", "vize.config.ts"].map((filename) => [
        filename,
        read(root, filename),
      ]),
    );
    const second = setupProject({
      root,
      runCommand(command) {
        assert.fail(
          `idempotent setup unexpectedly ran ${command.command} ${command.args.join(" ")}`,
        );
      },
    });

    assert.deepEqual(second.createdFiles, []);
    assert.equal(second.migratedViteConfig, null);
    assert.equal(second.enabledVitePlusLint, false);
    for (const [filename, source] of firstSources) {
      assert.equal(read(root, filename), source, `${filename} changed on the second setup`);
    }
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("preserves custom configs and scripts instead of guessing migrations", () => {
  const root = temporaryProject("no-clobber");
  try {
    const vizeConfig = `{"compiler":{"vapor":true}}\n`;
    const oxlintConfig = `{\n  // custom rules stay byte-for-byte intact\n  "rules": { "no-console": "error" }\n}\n`;
    const viteConfig = `import { defineConfig } from "vite-plus";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue({ template: { compilerOptions: { whitespace: "preserve" } } })],
});
`;
    write(
      root,
      "package.json",
      `{
\t"name": "custom-fixture",
\t"scripts": {
\t\t"vize:check": "vize check --tsconfig tsconfig.app.json"
\t}
}
`,
    );
    write(root, "vize.config.json", vizeConfig);
    write(root, ".oxlintrc.jsonc", oxlintConfig);
    write(root, "vite.config.ts", viteConfig);

    const result = setupProject({ root, install: false });

    assert.deepEqual(result.preservedFiles, [
      "vize.config.json",
      "vite.config.ts",
      ".oxlintrc.jsonc",
    ]);
    assert.deepEqual(result.preservedScripts, ["vize:check"]);
    assert.equal(read(root, "vize.config.json"), vizeConfig);
    assert.equal(read(root, ".oxlintrc.jsonc"), oxlintConfig);
    assert.equal(read(root, "vite.config.ts"), viteConfig);
    const scripts = packageJson(root).scripts as Record<string, string>;
    assert.equal(scripts["vize:check"], "vize check --tsconfig tsconfig.app.json");
    assert.equal(scripts["vize:ready"], "vize ready src");
    assert.match(read(root, "package.json"), /^\t"scripts"/mu);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("does not write any setup file when package.json is invalid", () => {
  const root = temporaryProject("invalid-package");
  try {
    write(root, "package.json", "{ invalid");
    const before = read(root, "package.json");

    assert.throws(() => setupProject({ root, install: false }), /Invalid package\.json/u);
    assert.equal(read(root, "package.json"), before);
    assert.deepEqual(fs.readdirSync(root), ["package.json"]);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("reports files written before a later atomic write fails", () => {
  const root = temporaryProject("partial-write");
  try {
    write(root, "package.json", `{"name":"partial-write-fixture","private":true}\n`);
    let writes = 0;

    assert.throws(
      () =>
        setupProject({
          root,
          install: false,
          writeFile(filename, source) {
            writes += 1;
            if (writes === 2) {
              throw new Error("simulated disk failure");
            }
            fs.writeFileSync(filename, source);
          },
        }),
      /Setup partially completed: wrote vize\.config\.ts before oxlint\.config\.ts failed/u,
    );
    assert.ok(fs.existsSync(path.join(root, "vize.config.ts")));
    assert.equal(fs.existsSync(path.join(root, "oxlint.config.ts")), false);
    assert.deepEqual(packageJson(root), { name: "partial-write-fixture", private: true });
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("preserves an existing Vite+ lint block without creating a second config", () => {
  const root = temporaryProject("vite-plus-lint");
  try {
    write(root, "package.json", `{"name":"lint-fixture","private":true}\n`);
    const viteConfig = `import { defineConfig } from "vite-plus";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [vize()],
  lint: {
    rules: {
      "no-console": "error",
    },
  },
});
`;
    write(root, "vite.config.ts", viteConfig);

    const result = setupProject({ root, install: false });

    assert.equal(read(root, "vite.config.ts"), viteConfig);
    assert.equal(fs.existsSync(path.join(root, "oxlint.config.ts")), false);
    assert.equal(result.enabledVitePlusLint, false);
    assert.ok(result.preservedFiles.includes("Vite+ lint configuration"));
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("does not mistake an oxlint plugin comment in plain Vite for a working config", () => {
  const root = temporaryProject("plain-vite-comment");
  try {
    write(root, "package.json", `{"name":"plain-vite-fixture","private":true}\n`);
    const viteConfig = `import { defineConfig } from "vite";

// oxlint-plugin-vize should be configured separately in a plain Vite project.
export default defineConfig({});
`;
    write(root, "vite.config.ts", viteConfig);

    const result = setupProject({ root, install: false });

    assert.equal(read(root, "vite.config.ts"), viteConfig);
    assert.equal(result.enabledVitePlusLint, false);
    assert.match(read(root, "oxlint.config.ts"), /from "oxlint-plugin-vize"/u);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("preserves multiline Vite+ imports when lint injection has no safe anchor", () => {
  const root = temporaryProject("multiline-import");
  try {
    write(root, "package.json", `{"name":"multiline-import-fixture","private":true}\n`);
    const viteConfig = `import {
  defineConfig,
} from "vite-plus";

export default defineConfig({});
`;
    write(root, "vite.config.ts", viteConfig);

    const result = setupProject({ root, install: false });

    assert.equal(read(root, "vite.config.ts"), viteConfig);
    assert.equal(result.enabledVitePlusLint, false);
    assert.ok(result.preservedFiles.includes("Vite+ lint configuration"));
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});

test("the packed CLI configures a strict temporary project without native loading", () => {
  const root = temporaryProject("packed-cli");
  try {
    write(root, "package.json", `{"name":"packed-fixture","private":true}\n`);
    write(
      root,
      "vite.config.mjs",
      `import vuePlugin from '@vitejs/plugin-vue'
export default { plugins: [vuePlugin()] }
`,
    );
    const packageDir = path.dirname(fileURLToPath(import.meta.url));
    const cli = path.join(packageDir, "../dist/cli.mjs");

    const output = execFileSync(process.execPath, [cli, "setup", root, "--no-install"], {
      encoding: "utf8",
    });

    assert.match(output, /\[vize setup\] created vize\.config\.ts/u);
    assert.match(output, /install dependencies with: vp add -D/u);
    assert.match(output, /configuration written; install dependencies before running Vize/u);
    assert.match(read(root, "vite.config.mjs"), /@vizejs\/vite-plugin/u);
    assert.ok(fs.existsSync(path.join(root, "oxlint.config.ts")));
    const scripts = packageJson(root).scripts as Record<string, string>;
    assert.equal(scripts["vize:ready"], "vize ready src");

    const invalid = spawnSync(process.execPath, [cli, "setup", root, "--bogus"], {
      encoding: "utf8",
    });
    assert.notEqual(invalid.status, 0);
    assert.match(invalid.stderr, /\[vize\] Unknown setup option: --bogus/u);
  } finally {
    fs.rmSync(root, { force: true, recursive: true });
  }
});
