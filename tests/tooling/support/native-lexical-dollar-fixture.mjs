// Nonexecuted byte fixtures exercise receipt refusals; they are never native evidence.
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import {
  digest,
  cargoArgs,
  sourcePaths,
  originalReceiptNames,
  unobserved,
} from "../../../tools/support/compat/github/native-lexical-dollar-metadata.mjs";
import {
  nativeLaw,
  producerLaws,
} from "../../../tools/support/compat/github/native-lexical-dollar-receipt.mjs";

const actualRoot = fs.realpathSync(new URL("../../../", import.meta.url));
const corpusPath = "tests/_fixtures/differential/typechecker/lexical-dollar-slot-6879";
const json = (value) => `${JSON.stringify(value, null, 2)}\n`;
export const write = (root, file, body) => {
  const target = path.join(root, file);
  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.writeFileSync(target, body);
};
export const editJson = (root, file, edit) => {
  const value = JSON.parse(fs.readFileSync(path.join(root, file)));
  edit(value);
  write(root, file, json(value));
};
const identity = (file) => {
  const bytes = fs.readFileSync(file);
  return { path: file, byteLength: bytes.length, sha256: digest(bytes) };
};

export function createPacketFixture() {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-dollar-packet-unit-")));
  const repo = path.join(root, "repository");
  fs.mkdirSync(repo);
  const git = (args) => execFileSync("git", ["--no-replace-objects", ...args], { cwd: repo });
  const manifestPath = "tests/performance/support/warm-type-backed-lexical-dollar-manifest.json";
  const manifest = JSON.parse(fs.readFileSync(path.join(actualRoot, manifestPath)));
  const files = sourcePaths(manifest);
  for (const file of files) write(repo, file, fs.readFileSync(path.join(actualRoot, file)));
  git(["init", "--quiet"]);
  git(["add", "."]);
  git([
    "-c",
    "user.name=Receipt fixture",
    "-c",
    "user.email=fixture@invalid.example",
    "commit",
    "--quiet",
    "-m",
    "nonexecuted receipt parser inputs",
  ]);
  const head = git(["rev-parse", "HEAD"]).toString().trim();
  const tree = git(["rev-parse", "HEAD^{tree}"]).toString().trim();
  const copied = files.map((file) => {
    const bytes = fs.readFileSync(path.join(repo, file));
    write(root, `source/${file}`, bytes);
    return {
      file,
      byteLength: bytes.length,
      sha256: digest(bytes),
      gitEntryHex: git(["ls-tree", "-z", "HEAD", "--", file]).toString("hex"),
    };
  });
  fs.cpSync(path.join(actualRoot, corpusPath), path.join(root, "corpus"), { recursive: true });
  write(root, "source.sha", `${head}\n`);
  const cargoTarget = path.join(root, "target");
  const testBinaryPath = path.join(cargoTarget, "ci-opt/deps/vize_canon-unit-fixture");
  write(root, path.relative(root, testBinaryPath), "nonexecuted test binary fixture\n");
  const packagePath = path.join(root, "runtime/package.json");
  const packageBytes = Buffer.from(
    json({
      name: `@typescript/typescript-${process.platform}-${process.arch}`,
      version: "7.0.2",
      fixture: "not executed",
    }),
  );
  write(root, "runtime/package.json", packageBytes);
  const nativePath = path.join(root, "runtime/lib/tsc");
  write(root, "runtime/lib/tsc", "nonexecuted native fixture\n");
  const native = {
    ...identity(nativePath),
    packagePath,
    packageBytes: [...packageBytes],
    versionProbe: {
      stdoutBytes: [...Buffer.from("Version 7.0.2\n")],
      stderrBytes: [],
      status: 0,
      signal: null,
    },
  };
  const compilerArtifact = {
    reason: "compiler-artifact",
    target: {
      name: "vize_canon",
      kind: ["lib"],
      src_path: path.join(repo, "crates/vize_canon/src/lib.rs"),
    },
    profile: { test: true, opt_level: "3", debug_assertions: false },
    features: ["default", "native"],
    executable: testBinaryPath,
    fresh: true,
  };
  write(
    root,
    "cargo.stdout",
    `${JSON.stringify(compilerArtifact)}\n${JSON.stringify({ reason: "build-finished", success: true })}\n`,
  );
  write(root, "cargo.stderr", "");
  write(root, "cargo.status", "0\n");
  write(root, "cargo.argv.nul", [...cargoArgs, ""].join("\0"));
  const original = {
    sourceSha: head,
    matchedCompleteExpectations: true,
    cases: [
      "imported_callable_default_remains_defined_in_all_template_prop_views",
      "undefined_callable_default_keeps_original_cli_conditions_optional",
      "undefined_callable_default_still_rejects_unguarded_template_calls",
    ].map((name) => ({
      name,
      sourceSha: head,
      nativeBinary: nativePath,
      nativeSha256: native.sha256,
      nativePackageSha256: digest(packageBytes),
      nativeVersion: "Version 7.0.2",
    })),
  };
  const originalReceipts = originalReceiptNames.map((name) => {
    const body = json(
      name === "receipts.json" ? original : { sourceSha: head, nativeBinary: nativePath },
    );
    write(root, `original-receipts/${name}`, body);
    write(root, `original-runtime/${name}`, body);
    return { name, ...identity(path.join(root, "original-receipts", name)) };
  });
  write(
    root,
    "vue/package.json",
    json({ name: "vue", version: "3.6.0-beta.10", fixture: "not executed" }),
  );
  const vueManifest = identity(path.join(root, "vue/package.json"));
  const metadata = {
    version: 1,
    source: { root: repo, head, tree, noReplaceObjects: true },
    copied,
    testBinary: identity(testBinaryPath),
    compilerArtifact,
    originalReceipts,
    buildTools: ["cargo", "rustc"].map((tool) => ({
      tool,
      stdout: `${tool} 1.99.0 (nonexecuted fixture)\n`,
      status: 0,
      signal: null,
    })),
    vueManifest,
    runtimeEnvironment: {
      requiredNative: "1",
      disableNative: null,
      tsgoPath: nativePath,
      corsaPath: nativePath,
      cargoTarget,
    },
    native,
    unobserved,
  };
  write(root, "metadata.json", json(metadata));
  for (const phase of ["producer", "native"]) {
    const names = phase === "producer" ? producerLaws : [nativeLaw];
    write(
      root,
      `${phase}.stdout`,
      `${names.map((name) => `test ${name} ... ok`).join("\n")}\ntest result: ok. ${names.length} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;\n`,
    );
    write(root, `${phase}.stderr`, "");
    write(root, `${phase}.status`, "0\n");
    const selector =
      phase === "producer" ? "virtual_ts::scope::globals::instance::tests::" : nativeLaw;
    write(
      root,
      `${phase}.argv.nul`,
      [
        testBinaryPath,
        selector,
        ...(phase === "native" ? ["--exact"] : []),
        "--nocapture",
        "--test-threads=1",
        "",
      ].join("\0"),
    );
  }
  const scopeBody = fs.readFileSync(
    path.join(
      repo,
      "crates/vize_canon/src/virtual_ts/scope/globals/instance/tests/lexical_scope.rs",
    ),
    "utf8",
  );
  const template = /const TEMPLATE: &str = r#"([\s\S]+?)"#;/u.exec(scopeBody)[1];
  for (const strict of [false, true])
    write(
      root,
      `producer/producer-scopes-${strict}.json`,
      json({
        strictInstanceGlobals: strict,
        script: "const label = 'x'\n",
        template,
        templateOffset: 1000,
        completeSummary: "nonexecuted parser fixture",
        expressions: [{ fixture: true }],
        scopes: [{ fixture: true }],
      }),
    );
  const sourceMap = [
    "CompositeSourceMap {",
    " sfc_map: Some(SfcSourceMap { projection: ProjectionMapping {",
    " spans: [VizeMapping { gen_range: 0..1, src_range: 0..1, sub_spans: [], }],",
    " meta: [], semantic_links: [], prop_default_key_links: [],",
    " authored_base: 0, authored_disjoint: false, }, blocks: [",
    " SfcBlockRange { start: 0, end: 1, block_type: ScriptSetup, },",
    " SfcBlockRange { start: 1, end: 2, block_type: Template, }, ] }),",
    " import_map: ImportSourceMap { adjustments: [], },",
    "}",
  ].join("\n");
  const expected = JSON.parse(fs.readFileSync(path.join(root, "corpus/expected.json")));
  const sourceFiles = [
    "valid",
    "wrong",
    "wrong",
    "escape",
    "foreign",
    "loop-source",
    "globals",
    "globals",
    "valid",
  ];
  const observations = expected.map((value, i) => {
    const source = (file) => fs.readFileSync(path.join(root, "corpus", file), "utf8");
    const inputs = [
      { file: "src/Panel.vue", source: source("Panel.vue.txt") },
      { file: "src/App.vue", source: source(`${sourceFiles[i]}.vue.txt`) },
      { file: "tsconfig.json", source: source("case.tsconfig.json.txt") },
    ];
    const projectRoot = path.join(root, `authored/${i}`);
    const virtualRoot = path.join(root, `virtual/${i}`);
    const requestedConfigPath = path.join(virtualRoot, "tsconfig.json");
    const config = {
      compilerOptions: {
        ...JSON.parse(inputs[2].source).compilerOptions,
        allowImportingTsExtensions: true,
        noUncheckedSideEffectImports: false,
      },
      exclude: [],
      include: [
        "__vize_helpers.d.ts",
        "__vize_vue_modules.d.ts",
        "src/App.vue.ts",
        "src/Panel.vue.ts",
      ],
    };
    const generatedFiles = [inputs[1], inputs[0]].map((input) => ({
      original: path.join(projectRoot, input.file),
      virtual: path.join(virtualRoot, `${input.file}.ts`),
      authoredSource: input.source,
      code: "abc",
      sourceMap: !input.source.startsWith('<script setup lang="ts"></script>')
        ? sourceMap
        : sourceMap.replace(" SfcBlockRange { start: 0, end: 1, block_type: ScriptSetup, },\n", ""),
    }));
    const packet = {
      route: "native-cli",
      completion: "REAPED",
      nativeBinary: nativePath,
      nativeSha256: native.sha256,
      nativePid: null,
      projectRoot,
      virtualRoot,
      actualCwd: virtualRoot,
      actualArgs: [
        "--checkers",
        "1",
        "--incremental",
        "--tsBuildInfoFile",
        path.join(virtualRoot, ".vize-buildinfo", "tsconfig.tsbuildinfo"),
        "--pretty",
        "false",
        "--project",
        requestedConfigPath,
      ],
      requestedConfigPath,
      requestedConfigBytes: json(config),
      authoredConfigPath: path.join(projectRoot, "tsconfig.json"),
      authoredConfigBytes: inputs[2].source,
      generatedFiles,
      status: `exit status: ${value.exitCode}`,
      exitCode: value.exitCode,
      success: value.success,
      rawStdoutBytes: [
        ...Buffer.from(
          value.diagnostics
            .map(
              (d) =>
                `${generatedFiles[0].virtual}(1,1): error TS${d.code}: nonexecuted packet parser fixture\n`,
            )
            .join(""),
        ),
      ],
      rawStderrBytes: [],
      unobserved: [
        "native PID",
        "native engine normalized options",
        "native engine loaded source-file vector",
      ],
    };
    write(root, `native/${String(i).padStart(3, "0")}-native-cli.json`, json(packet));
    return {
      ...value,
      strictInstanceGlobals: i !== 2 && i !== 7,
      inputs,
      projectRoot,
      nativeBinary: nativePath,
      nativeSha256: native.sha256,
      vueManifestPath: vueManifest.path,
      vueManifestBytes: fs.readFileSync(vueManifest.path, "utf8"),
    };
  });
  write(root, "native/normal-batch-results.json", json(observations));
  return { root, metadata, cleanup: () => fs.rmSync(root, { recursive: true }) };
}
