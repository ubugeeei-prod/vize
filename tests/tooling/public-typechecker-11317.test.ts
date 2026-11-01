import assert from "node:assert/strict";
import { realpathSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { writeTarGz } from "../../tools/support/archive/tar.ts";
import { createHash } from "node:crypto";
import {
  cases,
  decodeInput,
  diagnostic,
  expected,
  input,
  profiles,
  sha256,
  sourcePins,
  sources,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/contract.ts";
import {
  gitBlob,
  validateCapsule,
  validateNative,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/authority.ts";
import {
  assertLocked,
  shippedImages,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/install.ts";
import {
  assessJson,
  childEnvironment,
  healthy,
  recorder,
  retain,
} from "../../tools/benchmarks/scripts/public-typechecker-11317/execute.ts";
import { argumentsFor } from "../../tools/benchmarks/scripts/public-typechecker-11317/run.ts";

const original = input();
const scratch = (action: (root: string) => void) => {
  const root = realpathSync(mkdtempSync(join(tmpdir(), "public-11317-pure-")));
  try {
    action(root);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
};

test("the five complete reporter files and settings remain pinned; only the role version changes", () => {
  assert.deepEqual(
    original.map((row) => [row.path, sha256(row.source)]),
    sourcePins,
  );
  assert.equal(original.length, 5);
  const before = sources(original, "before", "Original"),
    after = sources(original, "after", "Original");
  assert.deepEqual(before, original);
  assert.deepEqual(
    after.map((row) => ({
      ...row,
      source:
        row.path === "package.json"
          ? row.source.replace('"vize": "0.634.0"', '"vize": "0.632.0"')
          : row.source,
    })),
    original,
  );
  const packageJson = JSON.parse(before[0]!.source);
  assert.deepEqual(packageJson.devDependencies, {
    "typescript-native": "npm:typescript@7.0.2",
    vize: "0.632.0",
    vue: "3.5.44",
  });
  assert.deepEqual(packageJson.imports, { "#src/*": "./src/*.ts" });
  const config = JSON.parse(before[1]!.source);
  assert.deepEqual(config, {
    compilerOptions: {
      target: "ESNext",
      module: "ESNext",
      moduleResolution: "bundler",
      strict: true,
      noEmit: true,
      skipLibCheck: true,
    },
    include: ["src/**/*.ts", "src/**/*.vue"],
  });
  const raw = readFileSync(
    fileURLToPath(
      new URL(
        "../_fixtures/differential/typechecker/package-import-redirects-11317/input.json",
        import.meta.url,
      ),
    ),
  );
  assert.throws(() => decodeInput(Buffer.concat([raw, Buffer.from("\n")])));
});

test("all nine ordered transforms preserve lines, restore the exact errors, and retain the complete oracle", () => {
  assert.deepEqual(cases, [
    "Original",
    "NoVersion",
    "Relative",
    "NoVue",
    "VueImporter",
    "Repair",
    "RestoreOriginal",
    "VueImporterRepair",
    "VueImporterRestore",
  ]);
  assert.deepEqual(
    sources(original, "after", "RestoreOriginal"),
    sources(original, "after", "Original"),
  );
  assert.deepEqual(
    sources(original, "after", "VueImporterRestore"),
    sources(original, "after", "VueImporter"),
  );
  assert.equal(
    sources(original, "after", "NoVersion")[0]!.source,
    sources(original, "after", "Original")[0]!.source.replace('  "version": "0.0.0",\n', ""),
  );
  assert.equal(
    sources(original, "after", "Relative")[2]!.source,
    'import { value } from "./plain";\nexport const v2 = value;\n',
  );
  assert.equal(
    sources(original, "after", "NoVue")[3]!.source.split("\n")[4],
    'const probe: number = "not a number";',
  );
  for (const id of cases) {
    const files = sources(original, "after", id),
      oracle = expected(original, id);
    assert.deepEqual(Object.keys(oracle), [
      "files",
      "programs",
      "errorCount",
      "warningCount",
      "fileCount",
    ]);
    assert.equal(
      files[3]!.source.split("\n")[4],
      id === "Repair" || id === "VueImporterRepair"
        ? "const probe: number = 1;"
        : 'const probe: number = "not a number";',
    );
    const rows = oracle.files as { file: string; diagnostics: string[] }[];
    assert.deepEqual(
      rows.find((row) => row.file === "src/plain.ts")!.diagnostics,
      oracle.errorCount ? [diagnostic] : [],
    );
    assert.deepEqual(
      rows.flatMap((row) => (row.file === "src/plain.ts" ? [] : row.diagnostics)),
      [],
    );
    assert.equal(oracle.fileCount, id.startsWith("VueImporter") ? 4 : 3);
  }
});

function capsule() {
  const profile = profiles.after;
  const receipts = ["linux-x64", "win32-x64", "darwin-x64", "darwin-arm64"].map(
    (targetPlatform) => {
      const raw = Buffer.from(
        JSON.stringify({
          schema: "vize.platformVsix.v1",
          version: profile.version,
          source: profile.candidate,
          provider: {
            source: profile.candidate,
            releaseVersion: profile.version,
            targetPlatform,
            rustTarget: "x86_64-unknown-linux-gnu",
            nodeFile: "native.node",
            nativeBuildToken: "token",
            files: [{ path: "native.node", bytes: 3, sha256: "abc" }],
          },
        }),
      );
      return {
        rawBase64: raw.toString("base64"),
        sha256: sha256(raw),
        origin: { runId: Number(profile.run), artifactId: 1, member: targetPlatform + ".json" },
      };
    },
  );
  const entry = {
    integrity: "sha512-" + Buffer.alloc(64, 1).toString("base64"),
    tarball: "https://registry.npmjs.org/vize/-/vize.tgz",
    metadataOrigin: "https://registry.npmjs.org/vize/0.634.0",
  };
  return {
    version: profile.version,
    tagSourceSha: profile.candidate,
    producerSource: profile.source,
    receipts,
    npmSRI: { vize: entry, "@vizejs/native": entry, "@vizejs/native-linux-x64-gnu": entry },
  };
}

test("role/source/run and unique four-provider custody are closed; image and token mutations refuse", () => {
  const good = capsule();
  validateCapsule("after", good);
  validateNative("after", good, { bytes: 3, sha256: "abc" }, "token");
  assert.throws(() => validateCapsule("before", good));
  for (const key of ["producerSource", "tagSourceSha", "version"] as const) {
    assert.throws(() => validateCapsule("after", { ...good, [key]: "wrong" }));
  }
  assert.throws(() =>
    validateCapsule("after", { ...good, receipts: [...good.receipts, good.receipts[0]] }),
  );
  assert.throws(() => validateCapsule("after", { ...good, receipts: good.receipts.slice(1) }));
  const foreign = structuredClone(good);
  foreign.receipts[0]!.origin.runId++;
  assert.throws(() => validateCapsule("after", foreign));
  const mutated = structuredClone(good);
  mutated.receipts[0]!.rawBase64 = Buffer.from("changed").toString("base64");
  assert.throws(() => validateCapsule("after", mutated));
  assert.throws(() => validateNative("after", good, { bytes: 4, sha256: "abc" }, "token"));
  assert.throws(() => validateNative("after", good, { bytes: 3, sha256: "wrong" }, "token"));
  assert.throws(() => validateNative("after", good, { bytes: 3, sha256: "abc" }, "foreign"));
  const lock = {
    packages: {
      "node_modules/vize": {
        version: "0.634.0",
        ...good.npmSRI.vize,
        resolved: good.npmSRI.vize.tarball,
      },
    },
  };
  assertLocked(lock, good, "vize", "0.634.0");
  assert.throws(() =>
    assertLocked(
      { packages: { "node_modules/vize": { ...lock.packages["node_modules/vize"], link: true } } },
      good,
      "vize",
      "0.634.0",
    ),
  );
  assert.equal(gitBlob(Buffer.from("hello\n")), "ce013625030ba8dba906f756967f9e9ca394464a");
});

test("whole-SRI archives authenticate every shipped file without rejecting npm-owned extra bins", () =>
  scratch((root) => {
    const manifest = Buffer.from('{"name":"example","version":"1.0.0"}\n'),
      body = Buffer.from("雪🌸\r\n");
    const payload = writeTarGz([
      { name: "package/package.json", data: manifest },
      { name: "package/index.js", data: body },
    ]);
    const integrity = "sha512-" + createHash("sha512").update(payload).digest("base64");
    writeFileSync(join(root, "package.json"), manifest);
    writeFileSync(join(root, "index.js"), body);
    mkdirSync(join(root, "node_modules/.bin"), { recursive: true });
    writeFileSync(join(root, "node_modules/.bin/npm-owned"), "not shipped by this archive");
    assert.equal(shippedImages(root, payload, integrity).images.length, 2);
    assert.throws(() =>
      shippedImages(root, payload, "sha512-" + Buffer.alloc(64).toString("base64")),
    );
    writeFileSync(join(root, "index.js"), "mutated");
    assert.throws(() => shippedImages(root, payload, integrity));
  }));

const packet = (
  root: string,
  label: string,
  stdout: Buffer | null,
  stderr: Buffer | null,
  status: number | null,
  error?: Error,
) =>
  retain(
    recorder(root),
    label,
    ["node", "npx-cli.js", "vize", "check", "-f", "json"],
    root,
    "start",
    { pid: 123, status, signal: null, stdout, stderr, error },
  );
test("complete raw/null/UTF8/status2 failure data exists before semantic guards and no JSON fields disappear", () =>
  scratch((root) => {
    const oracle = expected(original, "Original"),
      bytes = Buffer.from(JSON.stringify(oracle));
    const good = packet(root, "good", bytes, Buffer.alloc(0), 1);
    assert.equal(assessJson(good, oracle, true).matchesAuthoredLaw, true);
    const extra = packet(
      root,
      "extra",
      Buffer.from(JSON.stringify({ ...oracle, unknownField: true })),
      Buffer.alloc(0),
      1,
    );
    assert.throws(() => assessJson(extra, oracle, true));
    assert.equal(
      JSON.parse(readFileSync(join(root, "extra.process.json"), "utf8")).stdout,
      extra.stdout,
    );
    const bad = packet(root, "status2", bytes, Buffer.from("backend failed"), 2);
    assert.throws(() => assessJson(bad, oracle, true));
    assert.deepEqual(readFileSync(join(root, "status2.stderr.raw")), Buffer.from("backend failed"));
    const missing = packet(root, "spawn", null, null, null, new Error("ENOENT"));
    assert.equal(missing.stdoutImage, null);
    assert.equal(missing.stdout, null);
    assert.throws(() => healthy(missing, 0));
    assert.ok(JSON.parse(readFileSync(join(root, "spawn.process.json"), "utf8")).error);
    const invalid = packet(root, "utf8", Buffer.from([255]), Buffer.alloc(0), 1);
    assert.ok(invalid.stdoutUtf8Error);
    assert.deepEqual(readFileSync(join(root, "utf8.stdout.raw")), Buffer.from([255]));
    const debt = packet(
      root,
      "debt",
      Buffer.from(JSON.stringify(expected(original, "Original", 0))),
      Buffer.alloc(0),
      0,
    );
    assert.equal(assessJson(debt, oracle, false).matchesAuthoredLaw, false);
    assert.throws(() => assessJson(debt, oracle, true));
  }));

test("no authority token reaches npm/native children and only exact arguments are accepted", () => {
  assert.deepEqual(
    childEnvironment({
      PATH: "/bin",
      GITHUB_TOKEN: "secret",
      GH_TOKEN: "secret",
      NODE_AUTH_TOKEN: "secret",
      ACTIONS_RUNTIME_TOKEN: "secret",
      password: "secret",
    }),
    { PATH: "/bin" },
  );
  assert.deepEqual(argumentsFor(["--out", "/a", "--npm", "/b", "--npx", "/c"]), {
    out: "/a",
    npm: "/b",
    npx: "/c",
  });
  assert.throws(() => argumentsFor(["--out", "relative", "--npm", "/b", "--npx", "/c"]));
  assert.throws(() => argumentsFor(["--out", "/a", "--npm", "/b", "--npm", "/c"]));
});

test("the new closed manual role preserves the old 0.602 job body and never selects it", () => {
  const workflow = readFileSync(
    fileURLToPath(
      new URL("../../.github/workflows/public-typechecker-consumer.yml", import.meta.url),
    ),
    "utf8",
  ).replace(/\r\n/gu, "\n"); // YAML checkout EOL only; no process/diagnostic normalization.
  assert.ok(workflow.includes("default: legacy-0.602"));
  assert.ok(
    workflow.includes(
      "if: github.event_name != 'workflow_dispatch' || inputs.consumer == 'legacy-0.602'",
    ),
  );
  const oldJob = workflow
    .slice(workflow.indexOf("  public:\n"), workflow.indexOf("  contracts-11317:\n"))
    .replace(
      "    if: github.event_name != 'workflow_dispatch' || inputs.consumer == 'legacy-0.602'\n",
      "",
    );
  assert.equal(sha256(oldJob), "08e442156bc782895a717f5a3d7fa7ce77bd7bc965202713fd3eab3e246370b1");
  assert.ok(workflow.includes("inputs.consumer == 'package-import-redirects-11317'"));
  const controls = workflow.slice(
    workflow.indexOf("  contracts-11317:\n"),
    workflow.indexOf("  public-11317:\n"),
  );
  const publicRole = workflow.slice(workflow.indexOf("  public-11317:\n"));
  for (const job of [controls, publicRole]) {
    assert.ok(job.includes("node-version: 24.22.1"), "each job pins Node");
    assert.ok(job.includes("runs-on: blacksmith-8vcpu-ubuntu-2404"), "registered runner");
  }
  assert.ok(publicRole.includes("permissions: { contents: read, actions: read }"));
  assert.ok(publicRole.includes("needs: contracts-11317"));
  assert.ok(publicRole.includes("github.repository == 'ubugeeei-prod/vize'"));
  assert.ok(publicRole.includes("npm@11.19.1"));
  assert.ok(publicRole.includes("- uses: actions/upload-artifact@"));
  assert.ok(publicRole.includes("        if: always()"));
  assert.ok(!workflow.slice(0, workflow.indexOf("  public-11317:\n")).includes("actions: read"));
  assert.ok(!workflow.includes("CORSA_PATH"));
});
