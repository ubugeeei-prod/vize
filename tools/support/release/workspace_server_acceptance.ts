import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { publicInputs } from "./public_acceptance/inputs.ts";
import { rejectOverrides } from "./public_acceptance/installed.ts";
import { derivePublicationPlan } from "./public_acceptance/plan.ts";
import { verifyPublication } from "./public_acceptance/registry.ts";

// Reported by @AndreyYolkin in #9201; #9202 repairs importer-scoped roots.
const fix = "4e9366ee86d31cc9c1c03961d83505e24ebd4163";
rejectOverrides();
for (const name of [
  "VIZE_BINDING_PATH",
  "VIZE_CORSA_DEV_PATHS",
  "LD_PRELOAD",
  "DYLD_INSERT_LIBRARIES",
])
  assert.ok(!process.env[name], `runtime override must be absent: ${name}`);
const output = path.resolve(process.env.PUBLIC_OUTPUT!);
fs.mkdirSync(output, { recursive: true });
const sha256 = (bytes: Buffer | string) => createHash("sha256").update(bytes).digest("hex");
const save = (name: string, value: unknown) =>
  fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
const write = (root: string, file: string, text: string) => {
  const target = path.join(root, file);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, text);
};

if (process.argv[2] === "prepare") {
  assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
  assert.equal(process.env.GITHUB_REF, "refs/heads/main");
  const source = publicInputs(process.env);
  const root = fs.realpathSync(process.cwd());
  execFileSync("git", ["--no-replace-objects", "merge-base", "--is-ancestor", fix, source.cut]);
  const plan = derivePublicationPlan(root, source.head);
  save("publication.json", await verifyPublication(plan, source));
  save("request.json", {
    source,
    fix,
    harnessSha256: sha256(fs.readFileSync(import.meta.filename)),
  });
  fs.appendFileSync(process.env.GITHUB_ENV!, `PUBLIC_VERSION=${plan.version}\n`);
} else {
  const version = process.env.PUBLIC_VERSION!;
  assert.match(version, /^0\.[1-9][0-9]*\.0$/u);
  const consumer = fs.realpathSync(
    fs.mkdtempSync(path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "vize-workspace-public-")),
  );
  assert.ok(!consumer.startsWith(fs.realpathSync(process.cwd()) + path.sep));
  function run(name: string, command: string, args: string[], cwd: string) {
    const started = new Date().toISOString();
    const result = spawnSync(command, args, {
      cwd,
      encoding: "utf8",
      timeout: 180_000,
      maxBuffer: 16 * 1024 * 1024,
    });
    const observation = {
      name,
      command,
      args,
      cwd,
      started,
      status: result.status,
      signal: result.signal,
      error: result.error?.message,
      stdout: result.stdout,
      stderr: result.stderr,
    };
    save(`${name}.json`, observation);
    return result;
  }
  write(
    consumer,
    "package.json",
    JSON.stringify({ private: true, dependencies: { pnpm: "10.33.0" } }),
  );
  const bootstrap = run(
    "pnpm-install",
    "npm",
    [
      "install",
      "--ignore-scripts",
      "--no-audit",
      "--no-fund",
      "--registry=https://registry.npmjs.org",
    ],
    consumer,
  );
  assert.equal(bootstrap.status, 0, bootstrap.stderr);
  const pnpm = path.join(consumer, "node_modules/pnpm/bin/pnpm.cjs");
  const root = path.join(consumer, "repro");
  write(root, "pnpm-workspace.yaml", "packages: [app, lib]\n");
  write(
    root,
    "package.json",
    JSON.stringify({
      name: "root",
      private: true,
      devDependencies: { vize: version, typescript: "5.9.3" },
    }),
  );
  write(
    root,
    "lib/package.json",
    JSON.stringify({
      name: "lib",
      private: true,
      type: "module",
      exports: { ".": "./src/index.ts" },
      dependencies: { "vue-i18n": "11.1.0" },
      peerDependencies: { vue: "3.5.0" },
    }),
  );
  write(
    root,
    "app/package.json",
    JSON.stringify({
      name: "app",
      private: true,
      type: "module",
      dependencies: { lib: "workspace:*", vue: "3.5.0" },
    }),
  );
  write(
    root,
    "app/tsconfig.json",
    JSON.stringify({
      compilerOptions: {
        strict: true,
        module: "ESNext",
        moduleResolution: "Bundler",
        target: "ES2022",
        skipLibCheck: true,
        noEmit: true,
      },
      include: ["src/**/*.ts", "src/**/*.vue"],
    }),
  );
  const library =
    "import { createI18n } from 'vue-i18n'\nexport const i18n = createI18n({ legacy: false, locale: 'en' })\n";
  write(root, "lib/src/index.ts", library);
  write(root, "app/src/main.ts", "import { i18n } from 'lib'\nexport default i18n\n");
  const install = run(
    "workspace-install",
    process.execPath,
    [pnpm, "install", "--registry=https://registry.npmjs.org"],
    root,
  );
  assert.equal(install.status, 0, install.stderr);
  const executable = fs.realpathSync(path.join(root, "node_modules/vize/bin/vize"));
  assert.ok(executable.startsWith(root + path.sep));
  const manifest = JSON.parse(
    fs.readFileSync(path.join(path.dirname(executable), "../package.json"), "utf8"),
  );
  assert.equal(manifest.name, "vize");
  assert.equal(manifest.version, version);
  const packageRequire = createRequire(path.join(path.dirname(executable), "../package.json"));
  const nativeManifest = packageRequire.resolve("@vizejs/native/package.json");
  const nativeRequire = createRequire(nativeManifest);
  const nativePlatform = `@vizejs/native-${process.platform}-${process.arch}${process.platform === "linux" ? "-gnu" : ""}`;
  const providers = [
    {
      name: "@vizejs/native",
      manifest: nativeManifest,
      entry: packageRequire.resolve("@vizejs/native"),
    },
    {
      name: nativePlatform,
      manifest: nativeRequire.resolve(`${nativePlatform}/package.json`),
      entry: nativeRequire.resolve(nativePlatform),
    },
    {
      name: `@typescript/typescript-${process.platform}-${process.arch}`,
      manifest: packageRequire.resolve(
        `@typescript/typescript-${process.platform}-${process.arch}/package.json`,
      ),
      entry: path.join(
        path.dirname(
          packageRequire.resolve(
            `@typescript/typescript-${process.platform}-${process.arch}/package.json`,
          ),
        ),
        "lib/tsc",
      ),
    },
  ].map((provider) => {
    const location = fs.realpathSync(provider.entry);
    assert.ok(location.startsWith(root + path.sep), provider.name);
    const metadata = JSON.parse(fs.readFileSync(provider.manifest, "utf8"));
    assert.equal(metadata.name, provider.name);
    if (provider.name.startsWith("@vizejs/")) assert.equal(metadata.version, version);
    return {
      name: provider.name,
      version: metadata.version,
      entry: location,
      sha256: sha256(fs.readFileSync(location)),
      manifestSha256: sha256(fs.readFileSync(provider.manifest)),
    };
  });
  const lock = fs.readFileSync(path.join(root, "pnpm-lock.yaml"));
  fs.copyFileSync(path.join(root, "pnpm-lock.yaml"), path.join(output, "pnpm-lock.yaml"));
  save("installed.json", {
    version,
    consumer,
    executable,
    executableSha256: sha256(fs.readFileSync(executable)),
    cliSha256: sha256(fs.readFileSync(path.join(path.dirname(executable), "../dist/cli.mjs"))),
    manifest,
    providers,
    lockSha256: sha256(lock),
    packageManager: "pnpm@10.33.0",
    node: process.version,
  });
  // The dependency must exist only in lib's lookup chain, as in the report.
  assert.ok(fs.existsSync(path.join(root, "lib/node_modules/vue-i18n/package.json")));
  assert.ok(!fs.existsSync(path.join(root, "app/node_modules/vue-i18n")));
  assert.ok(!fs.existsSync(path.join(root, "node_modules/vue-i18n")));
  const rows: {
    files: number;
    negative: boolean;
    server: string;
    status: number | null;
    signal: string | null;
    error?: string;
    parseError?: string;
    result?: {
      errorCount: number;
      fileCount: number;
      warningCount: number;
      files: { file: string; diagnostics: string[] }[];
    };
  }[] = [];
  for (const files of [3, 128]) {
    for (let index = 0; index < files; index++)
      write(root, `app/src/C${index}.vue`, "<template><div /></template>\n");
    for (const negative of [false, true]) {
      write(
        root,
        "lib/src/index.ts",
        library + (negative ? "const genuine: number = i18n\nvoid genuine\n" : ""),
      );
      for (const server of ["1", "2", "auto"]) {
        const args = [executable, "check", "src", "--no-config", "--format", "json"];
        if (server !== "auto") args.push("--servers", server);
        const result = run(
          `check-${files}-${negative ? "negative" : "positive"}-${server}`,
          process.execPath,
          args,
          path.join(root, "app"),
        );
        let parsed;
        let parseError;
        try {
          parsed = JSON.parse(result.stdout);
        } catch (error) {
          parseError = String(error);
        }
        rows.push({
          files,
          negative,
          server,
          status: result.status,
          signal: result.signal,
          error: result.error?.message,
          parseError,
          result: parsed,
        });
      }
    }
  }
  save("matrix.json", rows);
  // Assert after every original process has completed, retaining the entire
  // failing matrix as well as positive and real-error controls.
  for (const row of rows) {
    const label = `${row.files} files, negative=${row.negative}, servers=${row.server}`;
    assert.equal(row.error, undefined, label);
    assert.equal(row.signal, null, label);
    assert.equal(row.parseError, undefined, label);
    assert.ok(row.result, label);
    assert.equal(row.status, row.negative ? 1 : 0, label);
    assert.equal(row.result.errorCount, row.negative ? 1 : 0, label);
    assert.equal(row.result.fileCount, row.files + 2, label);
    assert.equal(row.result.warningCount, 0, label);
    if (row.negative) {
      const imported = row.result.files.find(
        (file) => path.resolve(root, "app", file.file) === path.join(root, "lib/src/index.ts"),
      );
      assert.ok(imported, label);
      assert.match(JSON.stringify(imported.diagnostics), /TS2322/u, label);
    }
    const single = rows.find(
      (other) =>
        other.files === row.files && other.negative === row.negative && other.server === "1",
    )!;
    assert.deepEqual(row.result, single.result, label);
  }
  save("success.json", {
    issue: 9201,
    fix,
    version,
    cases: rows.length,
    finished: new Date().toISOString(),
  });
  console.log(`Public vize ${version}: all ${rows.length} workspace server cases passed.`);
}
