/** Hosted-only current SDK4 observation over the unchanged sealed cold corpus. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { prepareColdScale, coldScaleSupportScriptFiles } from "./cold-typechecker-scale.ts";
import { coldScaleCommand } from "./cold-typechecker-scale-command.ts";
import { captureThenReplay } from "./cold-typechecker-scale-native-command.ts";
import { assertLinuxX64Image, type NativeImage } from "./cold-typechecker-scale-native-custody.ts";
import {
  currentProfileSource,
  verifyProfileSource,
} from "./cold-typechecker-scale-native-source.ts";

const { values } = parseArgs({
  options: {
    provider: { type: "string" },
    out: { type: "string" },
    project: { type: "string" },
    binary: { type: "string" },
    build: { type: "string" },
    "phase-binary": { type: "string" },
    "phase-build": { type: "string" },
    "cpu-profile": { type: "boolean", default: false },
  },
});
assert(values.provider && values.out && values.project && values.binary && values.build);
assert(values["phase-binary"] && values["phase-build"]);
assert.equal(process.platform, "linux");
assert.equal(process.arch, "x64");
const provider = fs.realpathSync(values.provider),
  out = path.join(
    fs.realpathSync(path.dirname(path.resolve(values.out))),
    path.basename(values.out),
  ),
  project = path.join(
    fs.realpathSync(path.dirname(path.resolve(values.project))),
    path.basename(values.project),
  );
const hash = (file: string) => createHash("sha256").update(fs.readFileSync(file)).digest("hex");
const lockSha = "9c2974672b5f5358e643276e6ec98c6084681ef609e8f25922ba4aa7adb0f0a6";
assert.equal(hash(path.join(provider, "package-lock.json")), lockSha);
const sourceClosure = Object.fromEntries(
  [values.build, values["phase-build"]].map((receipt, index) => {
    const build = JSON.parse(fs.readFileSync(receipt, "utf8")) as { sourceRoot: string };
    return [index === 0 ? "custody" : "normal", verifyProfileSource(build.sourceRoot, provider)];
  }),
);
fs.writeFileSync(
  path.join(path.dirname(out), "source-closure.json"),
  JSON.stringify(sourceClosure, null, 2) + "\n",
  { flag: "wx" },
);
const lock = JSON.parse(fs.readFileSync(path.join(provider, "package-lock.json"), "utf8")) as {
  packages: Record<string, { version: string; integrity?: string }>;
};
const images: NativeImage[] = [];
function knownImage(packagePath: string, relative: string | null, role: string, required: boolean) {
  const manifestFile = path.join(provider, packagePath, "package.json");
  if (!fs.existsSync(manifestFile)) {
    assert.equal(required, false);
    return;
  }
  const manifest = JSON.parse(fs.readFileSync(manifestFile, "utf8")) as {
    version: string;
    main?: string;
  };
  assert(lock.packages[packagePath]?.integrity, "image owner must belong to the frozen npm graph");
  assert.equal(manifest.version, lock.packages[packagePath].version);
  const entry = relative ?? manifest.main;
  assert(typeof entry === "string");
  if (relative === null) {
    const leaf = path.basename(entry);
    assert(entry.endsWith(".node") && (entry === leaf || entry === "./" + leaf));
  }
  const file = fs.realpathSync(path.join(provider, packagePath, entry));
  const root = fs.realpathSync(path.join(provider, packagePath));
  assert(!path.relative(root, file).startsWith(".."));
  const digest = hash(file);
  assertLinuxX64Image(file, digest);
  images.push({ path: file, sha256: digest, role });
}
// Finite Linux-x64 image owners in the authenticated597 provider lock; no binary auto-admission.
knownImage("node_modules/@typescript/typescript-linux-x64", "lib/tsc", "invoked-native", true);
knownImage(
  "node_modules/vize/node_modules/@typescript/typescript-linux-x64",
  "lib/tsc",
  "public-native",
  false,
);
knownImage("node_modules/@vizejs/native-linux-x64-gnu", null, "public-vize-gnu", false);
knownImage("node_modules/@vizejs/native-linux-x64-musl", null, "public-vize-musl", false);
knownImage("node_modules/@oxc-transform/binding-linux-x64-gnu", null, "public-oxc-gnu", false);
knownImage("node_modules/@oxc-transform/binding-linux-x64-musl", null, "public-oxc-musl", false);
const corpus = prepareColdScale(project, provider, 2500 + coldScaleSupportScriptFiles, false);
assert.equal(
  corpus.corpusSha256,
  "c6089b0bca8f5c3f23139cc16b0a75962501d124c2b613d877323c2816a078bc",
);
const { command, backend } = coldScaleCommand({
  provider,
  out,
  app: corpus.app,
  tool: "cli",
  binary: values.binary,
});
try {
  captureThenReplay({
    sourceSha: currentProfileSource.sha,
    sourceTree: currentProfileSource.tree,
    buildReceipt: path.resolve(values.build),
    phaseExecutable: path.resolve(values["phase-binary"]),
    phaseBuildReceipt: path.resolve(values["phase-build"]),
    providerLockSha256: lockSha,
    vizeExecutable: command.executable,
    vizeArgs: command.args,
    backend,
    backendSha256: "a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
    project,
    provider,
    cwd: command.cwd,
    out,
    expectedFiles: corpus.entries.map((entry) => path.join(project, entry.path)),
    metadataOnlyImages: images,
    profile: values["cpu-profile"],
  });
} catch (error) {
  fs.mkdirSync(out, { recursive: true });
  fs.writeFileSync(
    path.join(out, "controller-failure.json"),
    JSON.stringify(
      {
        message: String(error),
        stack: error instanceof Error ? error.stack : null,
        scope: "partial observations retained; no qualification",
      },
      null,
      2,
    ) + "\n",
    { flag: "wx" },
  );
  throw error;
}
