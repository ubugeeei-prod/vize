import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync, realpathSync } from "node:fs";
import { tmpdir } from "node:os";
import { resolve, join } from "node:path";
import test from "node:test";
import {
  authenticateCapsule,
  argumentsFor,
  collect,
  contract,
  hash,
  originalFiles,
  stateFiles,
  verdict,
  verifyInputs,
  type Result,
} from "../../../tools/support/release/verify_published/vite_startup.ts";
const target = contract.target as Record<string, string>;
const controls = contract.controls as Record<string, string>;
const ok: Result = { status: 0, signal: null, error: null, stdout: "", stderr: "" };
const failure = (stderr: string): Result => ({ ...ok, status: 1, stderr });
const source = (issue: number, state: string, path: string) =>
  stateFiles(issue, state).find((f) => f.path === path)!.source;
test("only Vize pins change in the fourteen original project files", () => {
  for (const issue of [11375, 11376]) {
    const originals = (
      contract.originals as { issue: number; files: { path: string; source: string }[] }[]
    ).find((row) => row.issue === issue)!;
    const files = originalFiles(issue);
    assert.equal(files.length, 7);
    for (let i = 0; i < 7; i++)
      assert.deepEqual(
        files[i],
        originals.files[i]!.path === "package.json"
          ? {
              ...originals.files[i],
              source: originals.files[i]!.source.replaceAll(
                issue === 11375 ? "0.640.0" : "0.641.0",
                target.version!,
              ),
            }
          : originals.files[i],
      );
  }
  assert.equal(JSON.parse(originalFiles(11375)[0]!.source).devDependencies.vite, "8.3.5");
  assert.equal(
    JSON.parse(originalFiles(11376)[0]!.source).devDependencies.vite,
    "npm:@voidzero-dev/vite-plus-core@1.4.0",
  );
});
test("closed build states preserve commands and exact source-owned mutations", () => {
  assert.deepEqual(argumentsFor(11375, "original-vize"), [
    "exec",
    "vite",
    "build",
    "-c",
    "vite.config.vize.ts",
  ]);
  assert.deepEqual(argumentsFor(11375, "original-official"), [
    "exec",
    "vite",
    "build",
    "-c",
    "vite.config.pluginvue.ts",
  ]);
  assert.equal(
    source(11375, "import-invalid", "src/main.ts"),
    controls.importPrefix + source(11375, "original-vize", "src/main.ts"),
  );
  assert.equal(
    source(11375, "import-repaired", "src/Placeholder.vue"),
    controls.repairedPlaceholder,
  );
  for (const state of [
    "original",
    "sentinel-canonical",
    "sentinel-ignore-override",
    "sentinel-scan-override",
  ])
    assert.deepEqual(argumentsFor(11376, state), ["exec", "vp", "build"]);
  assert.equal(source(11376, "sentinel-canonical", "src/Ignored.vue"), controls.missingSource);
  assert.equal(source(11376, "sentinel-canonical", "other/Outside.vue"), controls.missingSource);
  assert.equal(
    source(11376, "sentinel-canonical", "vite.config.ts"),
    source(11376, "original", "vite.config.ts"),
  );
  assert.equal(
    source(11376, "sentinel-ignore-override", "vite.config.ts"),
    source(11376, "original", "vite.config.ts").replace(
      '  ignores: ["src/Ignored.vue"],',
      '  ignores: ["src/Ignored.vue"],\n  vite: { ignorePatterns: [] },',
    ),
  );
  assert.throws(() => argumentsFor(11376, "scan-empty"));
});
test("actual spawn seam strips credentials and retains failure before verdict", () => {
  let retained = false;
  const mock = ((_cmd: string, _argv: string[], options: { env: NodeJS.ProcessEnv }) => {
    assert.equal(options.env.GITHUB_TOKEN, undefined);
    assert.equal(options.env.A_CUSTOM_SECRET, undefined);
    assert.equal(options.env.PATH, "/bin");
    return {
      status: 1,
      signal: null,
      error: undefined,
      pid: 8,
      stdout: Buffer.from("all stdout"),
      stderr: Buffer.from("all stderr"),
    };
  }) as typeof spawnSync;
  const result = collect(
    "/manager",
    ["exec", "vp", "build"],
    "/project",
    { GITHUB_TOKEN: "private", A_CUSTOM_SECRET: "private", PATH: "/bin" },
    (_row, stdout, stderr) => {
      assert.equal(stdout.toString(), "all stdout");
      assert.equal(stderr.toString(), "all stderr");
      retained = true;
    },
    mock,
  );
  assert(retained);
  assert.equal(result.status, 1);
  assert.throws(() => verdict(11376, "original", result, "/project"));
});
test("startup warnings cannot satisfy strict imported-file build failure", () => {
  verdict(11375, "original-vize", ok, "/project");
  verdict(
    11375,
    "import-invalid",
    failure("error during build:\nPlaceholder.vue: " + controls.missingBlock),
    "/project",
  );
  assert.throws(() =>
    verdict(
      11375,
      "import-invalid",
      failure("warn Placeholder.vue " + controls.missingBlock + "\nerror during build:\nunrelated"),
      "/project",
    ),
  );
  assert.throws(() => verdict(11375, "import-invalid", ok, "/project"));
});
test("scope probes require exact selected missing-source identities", () => {
  for (const [state, file] of [
    ["sentinel-ignore-override", "src/Ignored.vue"],
    ["sentinel-scan-override", "other/Outside.vue"],
  ]) {
    const message =
      controls.missingSourcePrefix +
      resolve("/project", file!.split("/")[0]!, "missing.html") +
      ") in " +
      resolve("/project", file!);
    verdict(11376, state!, failure(message), "/project");
    assert.throws(() =>
      verdict(
        11376,
        state!,
        failure(message.replace(file!.split("/")[1]!, "Unowned.vue")),
        "/project",
      ),
    );
    assert.throws(() => verdict(11376, state!, ok, "/project"));
  }
});
function capsule() {
  return {
    version: target.version,
    tagSourceSha: target.candidate,
    producerSource: target.producerSource,
    npmSRI: Object.fromEntries(
      ["vize", "@vizejs/vite-plugin", "@vizejs/native", "@vizejs/native-linux-x64-gnu"].map(
        (name) => [
          name,
          {
            integrity: "sha512-" + Buffer.alloc(64).toString("base64"),
            tarball: "https://registry.npmjs.org/payload.tgz",
            metadataOrigin: {},
          },
        ],
      ),
    ),
    receipts: ["darwin-arm64", "darwin-x64", "linux-x64", "win32-x64"].map((platform) => {
      const raw = Buffer.from(
        JSON.stringify({
          schema: "vize.platformVsix.v1",
          version: target.version,
          source: target.candidate,
          target: platform,
          provider: {
            schema: "vize.vscodeNativeProvider.v1",
            source: target.candidate,
            releaseVersion: target.version,
            rustTarget: "x86_64-unknown-linux-gnu",
            targetPlatform: platform,
          },
        }),
      );
      return {
        rawBase64: raw.toString("base64"),
        sha256: hash(raw),
        origin: {
          runId: Number(target.runId),
          artifactId: 1,
          member: "vize-" + platform + ".vsix.receipt.json",
        },
      };
    }),
  };
}
test("capsule binds selected four SRI objects and native receipt source/run", () => {
  authenticateCapsule(capsule());
  const wrong = capsule();
  wrong.receipts[0]!.origin.runId = 1;
  assert.throws(() => authenticateCapsule(wrong));
  const corrupt = capsule();
  corrupt.receipts[0]!.sha256 = "0".repeat(64);
  assert.throws(() => authenticateCapsule(corrupt));
  const duplicate = capsule();
  duplicate.receipts[0] = duplicate.receipts[1]!;
  assert.throws(() => authenticateCapsule(duplicate));
});
test("missing targets stay absent and authored bytes remain exact", () => {
  const root = mkdtempSync(join(tmpdir(), "published-startup-contract-"));
  try {
    const files = stateFiles(11376, "sentinel-canonical");
    for (const file of files) {
      const path = join(root, file.path);
      mkdirSync(join(path, ".."), { recursive: true });
      writeFileSync(path, file.source);
    }
    verifyInputs(root, files);
    writeFileSync(join(root, "src/missing.html"), "unexpected");
    assert.throws(() => verifyInputs(root, files));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
test("published mode needs same-run contracts while the source job remains explicit", () => {
  const workflow = readFileSync(
    new URL("../../../.github/workflows/viteplus-config-consumers.yml", import.meta.url),
    "utf8",
  );
  assert(workflow.includes("options: [source, published-startup-75-76]"));
  assert(workflow.includes("needs: published-contracts"));
  assert(
    workflow.includes(
      "github.event_name != 'workflow_dispatch' || inputs.mode != 'published-startup-75-76'",
    ),
  );
  assert(workflow.includes("uses: ./.github/workflows/full-source-cli.yml"));
  assert(workflow.includes("node --test tests/tooling/release/published-vite-startup.test.ts"));
});

test("pnpm12 manager executable follows its actual root bin declaration", async () => {
  const { managerRoute } =
    await import("../../../tools/support/release/verify_published/vite_startup_authority.ts");
  const root = mkdtempSync(join(tmpdir(), "pnpm-declared-bin-"));
  try {
    writeFileSync(
      join(root, "package.json"),
      JSON.stringify({ name: "pnpm", version: "12.16.1", bin: { pnpm: "pnpm" } }),
    );
    writeFileSync(join(root, "pnpm"), "declared executable fixture");
    assert.equal(managerRoute(root).executable, realpathSync(join(root, "pnpm")));
    assert.throws(() => managerRoute(join(root, "bin/pnpm.cjs")));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
