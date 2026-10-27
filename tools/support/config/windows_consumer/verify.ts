import assert from "node:assert/strict";
import {
  copyFileSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  asArray,
  asNumber,
  asRecord,
  asString,
  parseJson,
  readJsonRecord,
} from "../../json-value.ts";
import { cleanEnv, npmArgv, Recorder, sha256, success } from "./common.ts";
import type { Packet } from "./common.ts";

const [projectArg, evidenceArg, phase, version, authorityArg] = process.argv.slice(2);
assert.equal(process.env.GITHUB_ACTIONS, "true");
assert.equal(process.version, "v24.22.1");
assert.ok(projectArg && evidenceArg && version);
assert.ok(phase && ["before", "candidate", "public"].includes(phase));
assert.match(version, /^[0-9]+\.[0-9]+\.[0-9]+$/);
const project = resolve(projectArg),
  app = join(project, "apps/playground");
const record = new Recorder(evidenceArg);
const authority = authorityArg ? readJsonRecord(authorityArg) : {};
const base = cleanEnv();
const pinned = "86005b1a5d1266148e61b8969edaaaa73a7bfe53";
const commit = record.run("upstream-head", ["git", "rev-parse", "HEAD"], project);
success(commit);
assert.equal(commit.stdout.trim(), pinned);
success(
  record.run(
    "original-source-clean",
    [
      "git",
      "diff",
      "--exit-code",
      "HEAD",
      "--",
      "apps/playground/vite.config.ts",
      "apps/playground/src",
    ],
    project,
  ),
);
const config = join(app, "vite.config.ts");
const original = readFileSync(config, "utf8");
assert.ok(original.includes("import { defineConfig } from 'vite-plus'"));
assert.ok(original.includes("export default defineConfig({"));
mkdirSync(join(record.root, "inputs"), { recursive: true });
writeFileSync(join(record.root, "inputs/original-vite.config.ts.txt"), original);
cpSync(join(app, "src"), join(record.root, "inputs/original-src"), { recursive: true });
copyFileSync(
  join(app, "package.json"),
  join(record.root, "inputs/original-playground-package.json.txt"),
);
copyFileSync(
  join(project, "package.json"),
  join(record.root, "inputs/original-root-package.json.txt"),
);
copyFileSync(join(project, "pnpm-lock.yaml"), join(record.root, "original-pnpm-lock.yaml"));
const scripts = asRecord(readJsonRecord(join(app, "package.json")).scripts);
assert.equal(
  scripts["vize:lint:opinionated"],
  "vp exec vize lint --preset opinionated --help-level short src",
);
const vp = record.run("vp-version", ["vp", "--version"], project);
success(vp);
assert.match(vp.stdout + vp.stderr, /1\.4\.0/);
const pm = record.run("pnpm-version", ["vp", "exec", "pnpm", "--version"], project);
success(pm);
assert.equal(pm.stdout.trim(), "10.34.7");
const workspace = join(project, "pnpm-workspace.yaml");
const yaml = readFileSync(workspace, "utf8");
assert.ok(yaml.includes("overrides:\n"));
const hostSuffix =
  process.platform === "win32"
    ? "win32-x64-msvc"
    : process.platform === "darwin"
      ? "darwin-" + process.arch
      : "linux-x64-gnu";
const overrides: Record<string, string> = {
  vize: version,
  "@vizejs/native": version,
  vue: "3.5.35",
  "@vitejs/plugin-vue": "6.0.7",
};
if (phase === "candidate") {
  assert.equal(authority.kind, "source-image-sdk");
  assert.equal(authority.wrapperVersion, version);
  assert.equal(authority.source, process.env.SOURCE_SHA);
  overrides["@vizejs/native-" + hostSuffix] =
    "file:" + asString(authority.tarball).replaceAll("\\", "/");
}
const additions =
  Object.entries(overrides)
    .map(([name, value]) => "  " + JSON.stringify(name) + ": " + JSON.stringify(value))
    .join("\n") + "\n";
writeFileSync(workspace, yaml.replace("overrides:\n", "overrides:\n" + additions));
writeFileSync(join(record.root, "effective-pnpm-workspace.yaml"), readFileSync(workspace));
success(
  record.run(
    "install-selected-sdk",
    ["vp", "install", "--no-frozen-lockfile"],
    project,
    base,
    600_000,
  ),
);
copyFileSync(join(project, "pnpm-lock.yaml"), join(record.root, "effective-pnpm-lock.yaml"));
const identityScript = fileURLToPath(new URL("./identity.ts", import.meta.url));
const idPacket = record.run("installed-identity", [process.execPath, identityScript, app], project);
success(idPacket);
const identity = asRecord(parseJson(idPacket.stdout));
assert.equal(identity.node, "v24.22.1");
for (const key of ["cliVersion", "nativeVersion", "platformVersion"])
  assert.equal(identity[key], version, key);
assert.equal(identity.vueVersion, "3.5.35");
assert.equal(identity.pluginVueVersion, "6.0.7");
assert.equal(identity.vitePlusVersion, "1.4.0");
const native = asRecord(identity.identity),
  binary = asRecord(identity.binary);
assert.equal(native.kind, "native");
if (phase === "candidate") {
  assert.equal(native.token, authority.nativeBuildToken);
  assert.equal(binary.sha256, authority.nativeSha256);
  assert.equal(binary.bytes, authority.nativeBytes);
  assert.ok(
    readFileSync(join(project, "pnpm-lock.yaml"), "utf8").includes(asString(authority.integrity)),
  );
}
for (const name of ["vize", "@vizejs/native", "@vizejs/native-" + hostSuffix]) {
  const metadata = record.run(
    "metadata-" + name.replaceAll("/", "-").replaceAll("@", ""),
    npmArgv(
      "view",
      name + "@" + version,
      "dist",
      "--json",
      "--registry=https://registry.npmjs.org",
    ),
    project,
  );
  success(metadata);
  const dist = asRecord(parseJson(metadata.stdout));
  if (phase === "public") {
    const expected = asRecord(asRecord(authority.npmSRI)[name]);
    assert.equal(dist.integrity, expected.integrity);
    assert.equal(dist.tarball, expected.tarball);
    assert.equal(new URL(asString(expected.metadataOrigin)).hostname, "registry.npmjs.org");
  }
  if (!(phase === "candidate" && name === "@vizejs/native-" + hostSuffix))
    assert.ok(
      readFileSync(join(project, "pnpm-lock.yaml"), "utf8").includes(asString(dist.integrity)),
      "installed public package SRI: " + name,
    );
}
if (phase === "public") {
  assert.equal(authority.version, version);
  assert.equal(authority.tagSourceSha, process.env.PUBLIC_SOURCE_SHA);
  assert.match(asString(authority.producerSource), /^[0-9a-f]{40}$/);
  const producerAncestry = record.run(
    "producer-to-tag-ancestry",
    [
      "gh",
      "api",
      "repos/ubugeeei-prod/vize/compare/" +
        asString(authority.producerSource) +
        "..." +
        asString(authority.tagSourceSha),
    ],
    project,
  );
  success(producerAncestry);
  assert.ok(
    ["ahead", "identical"].includes(asString(asRecord(parseJson(producerAncestry.stdout)).status)),
  );
  for (const key of ["required10990MergeSha", "required10985MergeSha"])
    if (authority[key]) {
      const required = asString(authority[key]);
      const comparison = record.run(
        "ancestry-" + key,
        [
          "gh",
          "api",
          "repos/ubugeeei-prod/vize/compare/" +
            required +
            "..." +
            asString(authority.producerSource),
        ],
        project,
      );
      success(comparison);
      const parsed = asRecord(parseJson(comparison.stdout));
      assert.ok(["ahead", "identical"].includes(asString(parsed.status)));
      assert.equal(asRecord(parsed.base_commit).sha, required);
    }
  assert.ok(authority.required10990MergeSha);
  const receipts = asArray(authority.receipts).map((value) => asRecord(value));
  let matched = false;
  for (const item of receipts) {
    const origin = asRecord(item.origin);
    assert.ok(asNumber(origin.runId) > 0 && asNumber(origin.artifactId) > 0);
    assert.ok(asString(origin.member).length > 0);
    const raw = Buffer.from(asString(item.rawBase64), "base64");
    assert.equal(sha256(raw), item.sha256);
    const receipt = asRecord(parseJson(raw.toString("utf8")));
    assert.equal(receipt.schema, "vize.platformVsix.v1");
    assert.equal(receipt.version, version);
    assert.equal(receipt.source, authority.tagSourceSha);
    const provider = asRecord(receipt.provider);
    if (provider.targetPlatform !== identity.targetPlatform) continue;
    assert.equal(provider.source, authority.tagSourceSha);
    assert.equal(provider.releaseVersion, version);
    const files = asArray(provider.files)
      .map((value) => asRecord(value))
      .filter((value) => value.path === provider.nodeFile);
    assert.equal(files.length, 1);
    assert.equal(binary.bytes, files[0].bytes);
    assert.equal(binary.sha256, files[0].sha256);
    assert.equal(native.token, provider.nativeBuildToken);
    matched = true;
  }
  assert.equal(matched, true, "the actual installed image must match genuine producer bytes/token");
  record.json("public-authority.json", authority);
}
const probeVersion = record.run("cli-version", ["vp", "exec", "vize", "--version"], app);
success(probeVersion);
assert.equal(
  probeVersion.stdout.trim(),
  "vize " + (phase === "candidate" ? asString(authority.sourceVersion) : version),
);
function lint(label: string, args: string[]): Packet {
  const trace = join(record.root, "trace-" + label);
  mkdirSync(trace, { recursive: true });
  const preload = pathToFileURL(fileURLToPath(new URL("./trace-node.ts", import.meta.url))).href;
  return record.run(label, ["vp", "exec", "vize", ...args], app, {
    ...base,
    VIZE_WINDOWS_CONFIG_TRACE: trace,
    NODE_OPTIONS: [base.NODE_OPTIONS, "--import=" + preload].filter(Boolean).join(" "),
  });
}
function samePath(value: unknown, expected: string): boolean {
  if (typeof value !== "string") return false;
  const key = (path: string) => {
    const normal = resolve(path)
      .replaceAll("\\", "/")
      .replace(/^\/\/\?\//, "");
    return process.platform === "win32" ? normal.toLowerCase() : normal;
  };
  return key(value) === key(expected);
}
function transport(label: string, required: boolean): void {
  const directory = join(record.root, "trace-" + label);
  const frames = readdirSync(directory)
    .filter((name) => name.endsWith("-start.json"))
    .map((name) => readJsonRecord(join(directory, name)));
  const helpers = frames.filter((frame) =>
    asArray(frame.argv).some((value) => samePath(value, config)),
  );
  if (required) assert.ok(helpers.length > 0, "actual dynamic Node config child: " + label);
  for (const frame of helpers) {
    assert.equal(frame.node, "v24.22.1");
    const args = asArray(frame.argv);
    const execArgs = asArray(frame.execArgv);
    if (phase === "before") {
      const index = execArgs.indexOf("-e");
      assert.ok(index >= 0);
      assert.ok(asString(execArgs[index + 1]).length > 32_767);
    } else {
      assert.equal(execArgs.includes("-e"), false);
      const script = asRecord(frame.script);
      assert.equal(args[1], script.path);
      assert.equal(samePath(args[2], config), true);
      assert.equal(existsSync(asString(script.path)), false, "temporary runtime file is removed");
      const exit = readJsonRecord(join(directory, String(frame.pid) + "-exit.json"));
      assert.equal(
        samePath(asArray(exit.argv)[1], config),
        true,
        "eval-era logical argv is preserved",
      );
    }
  }
}
const originalPacket = lint("original-command", [
  "lint",
  "--preset",
  "opinionated",
  "--help-level",
  "short",
  "src",
]);
if (phase === "before" && process.platform === "win32") {
  assert.notEqual(originalPacket.status, 0);
  assert.match(
    originalPacket.stdout + originalPacket.stderr,
    /os error 206|filename or extension is too long/i,
  );
  record.json("acceptance.json", {
    phase,
    version,
    originalRepo: pinned,
    status: "expected-before-206",
    identity,
  });
  process.exit(0);
}
success(originalPacket);
transport("original-command", true);
if (phase === "candidate")
  success(
    record.run(
      "standalone-original-command",
      [
        asString(authority.cliBinary),
        "lint",
        "--preset",
        "opinionated",
        "--help-level",
        "short",
        "src",
      ],
      app,
    ),
  );
const policy = join(app, ".vize-10988-policy.ts");
const witness = join(app, "src/VizeConfigWitness.vue");
writeFileSync(
  witness,
  '<script setup lang="ts">\nconst payload = "<b>10988</b>";\n</script>\n<template><div v-html="payload"></div></template>\n',
);
writeFileSync(
  config,
  "import { vuePolicy } from './.vize-10988-policy.ts'\n" +
    original.replace(
      "export default defineConfig({",
      "export default defineConfig({\n  vue: vuePolicy,",
    ),
);
const configBytes = readFileSync(config);
for (const [label, severity] of [
  ["off-first", "off"],
  ["error", "error"],
  ["off-reloaded", "off"],
  ["throw", "off"],
] as const) {
  const marker = join(record.root, label + "-actual-config-node.json");
  const text =
    'import { writeFileSync } from "node:fs";\nwriteFileSync(' +
    JSON.stringify(marker) +
    ", JSON.stringify({phase:" +
    JSON.stringify(label) +
    ",node:process.version,execPath:process.execPath,argv:process.argv}));\n" +
    (label === "throw" ? 'throw new Error("VIZE_10988_IMPORTED_CONFIG_SENTINEL");\n' : "") +
    "export const vuePolicy = " +
    JSON.stringify({ linter: { rules: { "vue/no-v-html": severity } } }) +
    ";\n";
  writeFileSync(policy, text);
  writeFileSync(join(record.root, label + "-policy.ts.txt"), text);
  assert.equal(readFileSync(config).equals(configBytes), true, "only the imported module changes");
  const packet = lint(label, [
    "lint",
    "--preset",
    "opinionated",
    "--help-level",
    "short",
    "--format",
    "json",
    "src/VizeConfigWitness.vue",
  ]);
  assert.equal(
    readJsonRecord(marker).node,
    "v24.22.1",
    "actual config import Node, not the bootstrap Node",
  );
  transport(label, true);
  if (label === "throw") {
    assert.notEqual(packet.status, 0);
    assert.match(packet.stdout + packet.stderr, /VIZE_10988_IMPORTED_CONFIG_SENTINEL/);
  } else {
    assert.equal(packet.status, severity === "error" ? 1 : 0, packet.stdout + packet.stderr);
    const entries = asArray(parseJson(packet.stdout)).map((value) => asRecord(value));
    const entry = entries.find((value) => asString(value.file).endsWith("VizeConfigWitness.vue"));
    assert.ok(entry);
    const messages = asArray(entry.messages)
      .map((value) => asRecord(value))
      .filter((value) => value.ruleId === "vue/no-v-html");
    assert.equal(messages.length, severity === "error" ? 1 : 0);
    if (severity === "error") assert.equal(messages[0].severity, 2);
    assert.equal(asNumber(entry.errorCount), severity === "error" ? 1 : 0);
  }
}
writeFileSync(config, original);
record.json("acceptance.json", {
  phase,
  version,
  originalRepo: pinned,
  status: "pass",
  identity,
  sourceAuthority: authority,
  controls: ["original-command", "off-first", "error", "off-reloaded", "throw"],
  stdin: "untouched by observer; actual config projection and imported graph reload completed",
});
