// Client-only `nuxt generate` through Vite's Environment API must keep Nuxt's
// client manifest: stock Nuxt, the released adapter, and the candidate adapter.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { at, readJson } from "../../json-value.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const fixture = path.join(root, "tools/support/compat/nuxt/fixtures/nuxt-environment-api-generate");
// The component-mode reproduction already pins this Nuxt 4.6 cohort.
const dependencies = fs.realpathSync(
  path.join(root, "tools/support/compat/nuxt/fixtures/nuxt-component-modes/node_modules"),
);
const artifacts = path.resolve(
  process.argv[2] ?? path.join(os.tmpdir(), "vize-nuxt-environment-api-generate"),
);
const candidatePackages = new Map([
  ["nuxt", path.join(root, "npm/framework/nuxt/dist")],
  ["vite-plugin", path.join(root, "npm/builder/vite/dist")],
]);
const missingManifest = "Either manifest or precomputed data must be provided";

const versions: Record<string, string> = {};
for (const [name, version] of [
  ["nuxt", "4.6.4"],
  ["vite", "8.3.6"],
  ["@vizejs/nuxt", "0.691.0"],
] as const) {
  versions[name] = String(at(readJson(path.join(dependencies, name, "package.json")), "version"));
  assert.equal(versions[name], version);
}
fs.mkdirSync(artifacts, { recursive: true });

type Arm = {
  name: string;
  vize: boolean;
  candidate: boolean;
  environmentApi: boolean;
};

function prepareWorkspace(arm: Arm): string {
  const workspace = path.join(artifacts, arm.name);
  assert.equal(fs.existsSync(workspace), false, `${arm.name} workspace must start absent`);
  fs.cpSync(fixture, workspace, { recursive: true });
  const modules = path.join(workspace, "node_modules");
  fs.mkdirSync(path.join(modules, "@vizejs"), { recursive: true });
  for (const entry of fs.readdirSync(dependencies)) {
    if (entry === "@vizejs" || entry === ".cache") continue;
    fs.symlinkSync(path.join(dependencies, entry), path.join(modules, entry), "dir");
  }
  for (const entry of fs.readdirSync(path.join(dependencies, "@vizejs"))) {
    const installed = path.join(dependencies, "@vizejs", entry);
    const target = path.join(modules, "@vizejs", entry);
    const candidateDist = candidatePackages.get(entry);
    if (!arm.candidate || !candidateDist) {
      fs.symlinkSync(installed, target, "dir");
      continue;
    }
    assert.ok(fs.existsSync(path.join(candidateDist, "index.mjs")));
    fs.cpSync(installed, target, { recursive: true });
    fs.rmSync(path.join(target, "dist"), { recursive: true });
    fs.cpSync(candidateDist, path.join(target, "dist"), { recursive: true });
  }
  return workspace;
}

function generate(arm: Arm): { workspace: string; status: number | null; log: string } {
  const workspace = prepareWorkspace(arm);
  const logPath = path.join(artifacts, `${arm.name}-generate.log`);
  const logFd = fs.openSync(logPath, "w");
  let result: ReturnType<typeof spawnSync>;
  try {
    result = spawnSync(
      process.execPath,
      [path.join(workspace, "node_modules/nuxt/bin/nuxt.mjs"), "generate"],
      {
        cwd: workspace,
        env: {
          ...process.env,
          VIZE_ENABLED: String(arm.vize),
          VITE_ENVIRONMENT_API: String(arm.environmentApi),
          NO_COLOR: "1",
          NUXT_TELEMETRY_DISABLED: "1",
        },
        stdio: ["ignore", logFd, logFd],
        timeout: 300_000,
      },
    );
  } finally {
    fs.closeSync(logFd);
  }
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null, `${arm.name} generate must not be killed`);
  fs.rmSync(path.join(workspace, "node_modules"), { recursive: true, force: true });
  return { workspace, status: result.status, log: fs.readFileSync(logPath, "utf8") };
}

/** Public output with the per-build id and timestamps masked. */
function publicOutput(workspace: string): Map<string, string> {
  const mask = (text: string) =>
    text
      .replace(/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g, "<build-id>")
      .replace(/\b1\d{12}\b/g, "<timestamp>");
  const directory = path.join(workspace, ".output/public");
  const files = new Map<string, string>();
  for (const entry of fs.readdirSync(directory, { recursive: true, withFileTypes: true })) {
    if (!entry.isFile()) continue;
    const file = path.join(entry.parentPath, entry.name);
    files.set(mask(path.relative(directory, file)), mask(fs.readFileSync(file, "latin1")));
  }
  return new Map([...files].sort(([left], [right]) => left.localeCompare(right)));
}

function assertGenerated(arm: Arm, run: ReturnType<typeof generate>): Map<string, string> {
  assert.equal(run.status, 0, `${arm.name} generate must pass`);
  assert.doesNotMatch(run.log, /\[500\]/, `${arm.name} must prerender every route`);
  // Nuxt removes the Vite client manifest once its server-side copy is finalized.
  assert.equal(
    fs.existsSync(path.join(run.workspace, ".nuxt/dist/client/manifest.json")),
    false,
    `${arm.name} must finalize the client manifest`,
  );
  const output = publicOutput(run.workspace);
  for (const page of ["index.html", "200.html", "404.html"]) {
    assert.match(
      output.get(page) ?? "",
      /<script type="module" src="\/_nuxt\/[^"]+\.js"/,
      `${arm.name} ${page} must load the client entry from the manifest`,
    );
  }
  return output;
}

const stock = { name: "stock", vize: false, candidate: false, environmentApi: true };
const released = { name: "released", vize: true, candidate: false, environmentApi: true };
const classic = { name: "candidate-classic", vize: true, candidate: true, environmentApi: false };
const candidate = { name: "candidate", vize: true, candidate: true, environmentApi: true };

assertGenerated(stock, generate(stock));

const releasedRun = generate(released);
assert.notEqual(releasedRun.status, 0, "the released adapter must reproduce the failure");
assert.ok(releasedRun.log.includes(missingManifest), "released failure must be the lost manifest");

const classicOutput = assertGenerated(classic, generate(classic));
const candidateOutput = assertGenerated(candidate, generate(candidate));
assert.deepEqual(
  [...candidateOutput],
  [...classicOutput],
  "Environment API output must match the classic build of the same app",
);

fs.writeFileSync(
  path.join(artifacts, "proof.json"),
  JSON.stringify(
    {
      candidateHead: process.env.GITHUB_SHA ?? null,
      versions,
      arms: [stock, released, classic, candidate].map((arm) => arm.name),
      releasedFailure: missingManifest,
      publicFiles: [...candidateOutput.keys()],
    },
    null,
    2,
  ) + "\n",
);
console.log(
  "Nuxt Environment API generate keeps the client manifest and matches the classic build",
);
