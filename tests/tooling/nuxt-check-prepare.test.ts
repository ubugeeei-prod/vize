import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import * as fs from "node:fs";
import * as os from "node:os";
import * as path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import * as vm from "node:vm";
import ts from "typescript";

import { prepareNuxtCheckFixture } from "../_helpers/nuxt-check-prepare.ts";

function fixture(nuxt = true) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-checked-"));
  const sourceDir = path.join(root, "tests/_fixtures/_git/provider");
  const captureDir = path.join(root, "evidence");
  fs.mkdirSync(sourceDir, { recursive: true });
  fs.writeFileSync(
    path.join(sourceDir, "package.json"),
    JSON.stringify({
      name: "provider",
      dependencies: nuxt ? { nuxt: "4.4.8" } : { vue: "3.5.39" },
    }),
  );
  fs.writeFileSync(path.join(sourceDir, "pnpm-lock.yaml"), "authored lock\n");
  fs.writeFileSync(
    path.join(sourceDir, "nuxt.config.ts"),
    "export default { modules: ['./module.ts'] }\n",
  );
  fs.writeFileSync(path.join(sourceDir, "module.ts"), "export default () => {}\n");
  git(sourceDir, ["init", "-q"]);
  commit(sourceDir);
  const revision = git(sourceDir, ["rev-parse", "HEAD"]).trim();
  git(root, ["init", "-q"]);
  git(root, [
    "update-index",
    "--add",
    "--cacheinfo",
    `160000,${revision},tests/_fixtures/_git/provider`,
  ]);
  commit(root);
  return {
    root,
    sourceDir,
    captureDir,
    revision,
    options: { name: "provider", sourceDir, authorityRoot: root, captureDir },
    receipt: () =>
      JSON.parse(
        fs.readFileSync(path.join(captureDir, "provider/preparation/receipt.json"), "utf8"),
      ),
    cleanup: () => fs.rmSync(root, { recursive: true, force: true }),
  };
}

test("Nuxt preparation uses the literal checked root and retains unlocked resolution separately", () => {
  const item = fixture();
  const checkedRoot = path.join(item.root, "check-clean");
  fs.cpSync(item.sourceDir, checkedRoot, {
    recursive: true,
    filter: (file) => path.basename(file) !== ".git",
  });
  const invocations: Array<{ args: string[]; cwd: string }> = [];
  try {
    assert.equal(
      prepareNuxtCheckFixture({
        ...item.options,
        cwd: checkedRoot,
        run(args, options) {
          invocations.push({ args, cwd: options.cwd });
          if (args.includes("install")) {
            fs.writeFileSync(path.join(checkedRoot, "pnpm-lock.yaml"), "resolved lock with SRI\n");
            fs.mkdirSync(path.join(checkedRoot, "node_modules/nuxt"), { recursive: true });
            fs.writeFileSync(
              path.join(checkedRoot, "node_modules/nuxt/package.json"),
              '{"name":"nuxt","version":"4.4.8","types":"index.d.ts"}',
            );
            fs.writeFileSync(
              path.join(checkedRoot, "node_modules/nuxt/index.d.ts"),
              "export interface Provider {}\n",
            );
          } else {
            fs.mkdirSync(path.join(checkedRoot, ".nuxt/types"), { recursive: true });
            fs.writeFileSync(
              path.join(checkedRoot, ".nuxt/tsconfig.json"),
              '{"include":["./types/provider.d.ts"]}',
            );
            fs.writeFileSync(
              path.join(checkedRoot, ".nuxt/types/provider.d.ts"),
              "export interface Generated {}\n",
            );
          }
        },
      }),
      checkedRoot,
    );
    assert.deepEqual(invocations, [
      {
        args: [
          "-y",
          "pnpm@10",
          "install",
          "--no-frozen-lockfile",
          "--ignore-scripts",
          "--prefer-offline",
          "--ignore-workspace",
        ],
        cwd: checkedRoot,
      },
      { args: ["-y", "pnpm@10", "exec", "nuxt", "prepare"], cwd: checkedRoot },
    ]);
    assert.equal(
      fs.readFileSync(path.join(checkedRoot, "pnpm-lock.yaml"), "utf8"),
      "authored lock\n",
    );
    assert.equal(git(item.sourceDir, ["status", "--porcelain"]).trim(), "");
    const receipt = item.receipt();
    assert.equal(receipt.lockedInstallation, false);
    assert.equal(receipt.authoredPreserved, true);
    assert.equal(receipt.failed, false);
    assert.deepEqual(receipt.sourceMutations, []);
    assert.equal(receipt.lockChanges.length, 1);
    for (const role of [
      "authored-before",
      "resolved-install-lock",
      "installed-package-manifest",
      "installed-declaration-candidate",
      "nuxt-generated-provider",
    ]) {
      assert.ok(
        receipt.files.some((file: { role: string }) => file.role === role),
        role,
      );
    }
    const resolved = receipt.files.find(
      (file: { role: string }) => file.role === "resolved-install-lock",
    );
    assert.equal(
      fs.readFileSync(
        path.join(item.captureDir, "provider/preparation", resolved.retainedBytes),
        "utf8",
      ),
      "resolved lock with SRI\n",
    );
  } finally {
    item.cleanup();
  }
});

test("preparation rejects drift before running a provider", () => {
  const item = fixture();
  try {
    fs.appendFileSync(path.join(item.sourceDir, "module.ts"), "changed\n");
    let runs = 0;
    assert.throws(
      () =>
        prepareNuxtCheckFixture({
          ...item.options,
          run() {
            runs++;
          },
        }),
      /checked source differs from its pin/,
    );
    assert.equal(runs, 0);
  } finally {
    item.cleanup();
  }
});

test("a failed or source-mutating provider stays failed and restores every authored byte", () => {
  for (const mutation of [true, false, ...(process.platform === "win32" ? [] : ["mode"])]) {
    const item = fixture();
    try {
      let runs = 0;
      assert.throws(
        () =>
          prepareNuxtCheckFixture({
            ...item.options,
            run() {
              runs++;
              fs.writeFileSync(path.join(item.sourceDir, "pnpm-lock.yaml"), "resolved\n");
              if (mutation === "mode") fs.chmodSync(path.join(item.sourceDir, "module.ts"), 0o755);
              else if (mutation)
                fs.writeFileSync(path.join(item.sourceDir, "module.ts"), "mutated\n");
              else throw new Error("actual provider failed");
            },
          }),
        mutation ? /authored source changed|source mode changed/ : /actual provider failed/,
      );
      assert.equal(runs, 1, "do not prepare or retry after a failed install");
      assert.equal(git(item.sourceDir, ["status", "--porcelain"]).trim(), "");
      assert.equal(item.receipt().failed, true);
      assert.equal(item.receipt().authoredPreserved, true);
      assert.equal(fs.existsSync(path.join(item.sourceDir, ".vize-check-prepare-lock")), false);
    } finally {
      item.cleanup();
    }
  }
});

test("plain Vue has no Nuxt preparation or invented ImportMeta features", () => {
  const item = fixture(false);
  try {
    let runs = 0;
    assert.throws(
      () =>
        prepareNuxtCheckFixture({
          ...item.options,
          run() {
            runs++;
          },
        }),
      /no authored Nuxt provider/,
    );
    assert.equal(runs, 0);
    const file = path.join(item.root, "plain.ts");
    fs.writeFileSync(file, "export {};\nimport.meta.test;\nimport.meta.vfFeatures;\n");
    assert.deepEqual(
      diagnostics([file]).map(({ code, messageText }) => [
        code,
        ts.flattenDiagnosticMessageText(messageText, "\n"),
      ]),
      [
        [2339, "Property 'test' does not exist on type 'ImportMeta'."],
        [2339, "Property 'vfFeatures' does not exist on type 'ImportMeta'."],
      ],
    );
  } finally {
    item.cleanup();
  }
});

test("the pinned VueFes module's real prepare:types hooks retain boolean types and negative controls", async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-nuxt-hook-types-"));
  try {
    const fixturePath = path.resolve(
      path.dirname(fileURLToPath(import.meta.url)),
      "fixtures/vuefes-feature-flags.source",
    );
    const source = fs.readFileSync(fixturePath, "utf8");
    assert.equal(
      createHash("sha1")
        .update(`blob ${Buffer.byteLength(source)}\0`)
        .update(source)
        .digest("hex"),
      "dfccb61058aba2ade312fb667fc7b02183de9f7d",
    );
    const output = ts.transpileModule(source, {
      compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ESNext },
    }).outputText;
    const templates: Array<{ filename: string; getContents(): string }> = [];
    const hooks: Array<(payload: { references: Array<{ path: string }> }) => Promise<void>> = [];
    const exports: Record<string, any> = {};
    vm.runInNewContext(output, {
      exports,
      require(name: string) {
        assert.equal(name, "nuxt/kit");
        return {
          defineNuxtModule: (value: unknown) => value,
          addTemplate: (template: any) => templates.push(template),
        };
      },
    });
    await exports.default.setup(
      { timetable: true, soldOut: false },
      {
        options: {},
        hook(name: string, callback: any) {
          if (name === "prepare:types") hooks.push(callback);
        },
      },
    );
    const payload = { references: [] as Array<{ path: string }> };
    for (const hook of hooks) await hook(payload);
    assert.deepEqual(JSON.parse(JSON.stringify(payload)), {
      references: [
        { path: "./types/feature-flags.d.ts" },
        { path: "./types/nitro-feature-flags.d.ts" },
      ],
    });
    for (const template of templates) {
      const file = path.join(root, template.filename);
      fs.mkdirSync(path.dirname(file), { recursive: true });
      fs.writeFileSync(file, template.getContents());
    }
    const provider = path.join(root, "nuxt.d.ts");
    fs.writeFileSync(
      provider,
      payload.references
        .map((reference) => `/// <reference path="${reference.path}" />`)
        .join("\n"),
    );
    const app = path.join(root, "app.ts");
    fs.writeFileSync(
      app,
      "export {};\nconst flag: boolean = import.meta.vfFeatures.timetable;\nconst bad: string = import.meta.vfFeatures.soldOut;\nimport.meta.vfFeatures.missing;\nimport.meta.test;\n",
    );
    assert.deepEqual(
      diagnostics([app, provider]).map(({ code, messageText }) => [
        code,
        ts.flattenDiagnosticMessageText(messageText, "\n"),
      ]),
      [
        [2322, "Type 'boolean' is not assignable to type 'string'."],
        [2339, "Property 'missing' does not exist on type 'ImportMetaFeatureFlags'."],
        [2339, "Property 'test' does not exist on type 'ImportMeta'."],
      ],
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

function diagnostics(files: string[]): readonly ts.Diagnostic[] {
  return ts.getPreEmitDiagnostics(
    ts.createProgram(files, {
      target: ts.ScriptTarget.ESNext,
      module: ts.ModuleKind.ESNext,
      strict: true,
      skipLibCheck: true,
      noEmit: true,
      types: [],
    }),
  );
}

function git(cwd: string, args: string[]): string {
  return execFileSync("git", args, { cwd, encoding: "utf8" });
}

function commit(cwd: string): void {
  git(cwd, ["add", "."]);
  git(cwd, [
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.com",
    "commit",
    "-qm",
    "fixture",
  ]);
}
