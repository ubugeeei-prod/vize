/** Independent framework/source admission for P0#11020; all measurements are untimed. */
import assert from "node:assert/strict";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { componentCasingCases } from "./canon-component-casing-cases.ts";
import {
  nativeDiagnostics,
  position,
  sourceSdk,
  stockFrameworkProof,
  type NativeDiagnostic,
} from "./canon-component-casing-observe.ts";
import { identify, run, save, wrapperSource, hash } from "./canon-instance-directives-io.ts";
import { isRecord, jsonValue, packet, type Diagnostic } from "./canon-instance-directives-model.ts";
async function main() {
  const [providerArg, binaryArg, outArg] = process.argv.slice(2);
  assert.ok(providerArg && binaryArg && outArg);
  const provider = path.resolve(providerArg),
    binary = path.resolve(binaryArg),
    out = path.resolve(outArg);
  fs.mkdirSync(out);
  const rows = componentCasingCases();
  assert.equal(rows.length, 32);
  const sourceImage = identify(binary);
  const identities = [
    "package-lock.json",
    ...[
      "vize",
      "@vizejs/native",
      "@vizejs/native-linux-x64-gnu",
      "vue",
      "typescript",
      "typescript-native",
      "vue-tsc",
    ].map((name) => "node_modules/" + name + "/package.json"),
    "node_modules/vize/content-mapper-helpers.d.ts",
    "node_modules/vize/generated/content-mapper-types.d.ts",
    "node_modules/@vizejs/native-linux-x64-gnu/vize-vitrine.linux-x64-gnu.node",
  ].map((file) => ({ file, ...identify(path.join(provider, file)) }));
  const versions: Record<string, unknown> = {};
  for (const name of ["vize", "vue", "typescript", "typescript-native", "vue-tsc"]) {
    const m = jsonValue(
      fs.readFileSync(path.join(provider, "node_modules", name, "package.json"), "utf8"),
    );
    assert.ok(isRecord(m));
    versions[name] = m.version;
  }
  assert.deepEqual(versions, {
    vize: "0.602.0",
    vue: "3.5.42",
    typescript: "6.0.3",
    "typescript-native": "7.1.0-beta",
    "vue-tsc": "3.3.13",
  });
  const getter: unknown = await import(
    pathToFileURL(path.join(provider, "node_modules/typescript-native/lib/getExePath.js")).href
  );
  assert.ok(isRecord(getter) && typeof getter.default === "function");
  const checker: unknown = Reflect.apply(getter.default, getter, []);
  assert.ok(typeof checker === "string");
  const checkerImage = identify(checker);
  const wrapper = path.join(out, "tsgo");
  fs.writeFileSync(wrapper, wrapperSource, { mode: 0o755 });
  const config = {
    compilerOptions: {
      strict: true,
      noEmit: true,
      skipLibCheck: true,
      module: "ESNext",
      moduleResolution: "Bundler",
      target: "ES2024",
      lib: ["ES2024", "DOM"],
      types: [],
      incremental: false,
    },
    vueCompilerOptions: {
      strictTemplates: true,
      checkUnknownProps: true,
      checkUnknownComponents: false,
    },
    include: ["**/*.vue", "**/component.ts"],
  };
  const inputManifest = [];
  for (const role of ["published", "source"]) {
    const project = path.join(out, role);
    fs.mkdirSync(project);
    if (role === "source") sourceSdk(project, provider, binary);
    else
      fs.symlinkSync(
        path.join(provider, "node_modules"),
        path.join(project, "node_modules"),
        "dir",
      );
    for (const row of rows)
      for (const [file, text] of [
        ["component.ts", row.component],
        ["src/Tiny.vue", row.source],
      ]) {
        assert.ok(file && text);
        const relative = path.join(row.id, file),
          absolute = path.join(project, relative);
        fs.mkdirSync(path.dirname(absolute), { recursive: true });
        fs.writeFileSync(absolute, text);
        inputManifest.push({ role, file: relative, ...identify(absolute) });
      }
    save(path.join(project, "tsconfig.json"), config);
    save(path.join(project, "tsconfig.mapper.json"), {
      ...config,
      contentMappers: [{ package: "vize", extensions: [".vue"] }],
    });
  }
  const stock = stockFrameworkProof(provider, path.join(out, "published"), out);
  const published = path.join(out, "published"),
    vueFile = path.join(provider, "node_modules/vue-tsc/bin/vue-tsc.js");
  const vueTs = createRequire(vueFile).resolve("typescript/package.json");
  assert.equal(
    fs.realpathSync(vueTs),
    fs.realpathSync(path.join(provider, "node_modules/typescript/package.json")),
  );
  const official = run(
    out,
    "official-vue-tsc",
    process.execPath,
    [vueFile, "--pretty", "false", "--project", "tsconfig.json"],
    published,
  );
  const officialDiagnostics = nativeDiagnostics(official.stdout, official.status, published);
  const observations: {
    role: string;
    mode: string;
    diagnostics: (Diagnostic | NativeDiagnostic)[];
    sourceSdkManifest: unknown;
    nativeCalls: string[];
  }[] = [];
  for (const role of ["published", "source"]) {
    const project = path.join(out, role),
      cli = role === "source" ? binary : path.join(provider, "node_modules/vize/bin/vize");
    for (const mode of ["cli", "mapper"]) {
      const capture = path.join(out, "native", role, mode);
      fs.mkdirSync(capture, { recursive: true });
      const env = {
        ...process.env,
        VIZE_DIRECTIVE_NATIVE: checker,
        VIZE_DIRECTIVE_NATIVE_CAPTURE: capture,
      };
      const command = mode === "cli" ? cli : wrapper;
      const args =
        mode === "cli"
          ? [
              "check",
              "--no-config",
              "--format",
              "json",
              "--show-virtual-ts",
              "--tsconfig",
              "tsconfig.json",
              "--corsa-path",
              wrapper,
            ]
          : [
              "--runExternalCode",
              "--checkers",
              "1",
              "--noEmit",
              "--pretty",
              "false",
              "--project",
              "tsconfig.mapper.json",
            ];
      const result = run(out, role + "-" + mode, command, args, project, env);
      const cliResult = mode === "cli" ? packet(result.stdout, result.status, project) : null;
      const diagnostics =
        cliResult?.diagnostics ?? nativeDiagnostics(result.stdout, result.status, project);
      if (cliResult)
        for (const row of rows) {
          const file = cliResult.result.files.find(
            (file) =>
              path.relative(project, path.resolve(project, file.file)).replaceAll("\\", "/") ===
              row.id + "/src/Tiny.vue",
          );
          assert.ok(
            file && typeof file.virtualTs === "string" && file.virtualTs.length > 0,
            "complete source virtual TS required: " + role + "/" + row.id,
          );
        }
      const calls = fs.readdirSync(capture);
      assert.ok(calls.length > 0, "actual native child packets required");
      observations.push({
        role,
        mode,
        diagnostics,
        sourceSdkManifest:
          role === "source"
            ? jsonValue(
                fs.readFileSync(path.join(project, "node_modules/vize/package.json"), "utf8"),
              )
            : null,
        nativeCalls: calls,
      });
    }
  }
  const byCase = rows.map((row) => ({
    ...row,
    official: officialDiagnostics.filter((d) => d.file.startsWith(row.id + "/")),
    routes: observations.map((route) => ({
      role: route.role,
      mode: route.mode,
      diagnostics: route.diagnostics.filter((d) => d.file.startsWith(row.id + "/")),
    })),
  }));
  const summary = {
    schema: 1,
    issue: 11020,
    sourceImage,
    checkerImage,
    versions,
    identities,
    vueTs: identify(vueTs),
    physicalVue: fs.realpathSync(path.join(published, "node_modules/vue/package.json")),
    inputManifest,
    stock,
    byCase,
    timingUsable: false,
    claimedRelease: false,
  };
  save(path.join(out, "summary.json"), summary);
  const failures = [];
  for (const row of byCase) {
    const expected =
      row.expectedCode === null
        ? []
        : [
            {
              file: row.id + "/src/Tiny.vue",
              code: row.expectedCode,
              ...(row.expectedCode === 2322
                ? position(row.source, ":value=", 1)
                : position(row.source, "slotReport(item)")),
            },
          ];
    const checks = [
      { role: "official", mode: "vue-tsc", diagnostics: row.official },
      ...row.routes.filter(
        (r) =>
          r.role === "source" ||
          row.expectedCode === null ||
          [
            "pascal-pascal-invalid",
            "camel-camel-invalid",
            "pascal-pascal-poisoned",
            "camel-camel-poisoned",
          ].includes(row.id),
      ),
    ];
    for (const route of checks) {
      const actual = route.diagnostics.map((d) => ({
        file: d.file,
        code: d.code,
        line: d.line,
        column: d.column,
      }));
      try {
        assert.deepEqual(actual, expected);
      } catch {
        failures.push({ id: row.id, role: route.role, mode: route.mode, actual, expected });
      }
      for (const d of route.diagnostics) {
        if (d.code === 2322)
          assert.match(d.message, /^Type 'string' is not assignable to type 'number'\./u);
        if (d.code === 2345) assert.match(d.message, /^Argument of type /u);
      }
    }
  }
  const original = byCase.find((r) => r.id === "camel-kebab-invalid");
  assert.ok(original && original.source.includes(":value=\"'bad'\""));
  assert.equal(
    hash(original.source),
    "22149b752e86b665333b5e01798440de0a82ba9d89090a82636260d4a94c3d52",
  );
  assert.equal(
    hash(original.component),
    "de08d51f3208a6f063035048557b8c83f0f7b6ab9e4e7300573d1a89538c6902",
  );
  assert.deepEqual(
    original.routes.filter((r) => r.role === "published").map((r) => r.diagnostics),
    [[], []],
    "preserve the genuine published false-negative receipt",
  );
  for (const entry of identities)
    assert.deepEqual(identify(path.join(provider, entry.file)), {
      path: entry.path,
      bytes: entry.bytes,
      sha256: entry.sha256,
    });
  assert.deepEqual(identify(binary), sourceImage);
  assert.deepEqual(identify(checker), checkerImage);
  save(path.join(out, "failures.json"), failures);
  assert.deepEqual(
    failures,
    [],
    "every source and independent framework full packet must meet the exact token oracle",
  );
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url))
  await main();
