import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { rejectOverrides } from "../public_acceptance/installed.ts";
import { publicReader } from "../public_acceptance/http.ts";
import { captureCommand, effectiveDenoConfig, publicEnvironment, strictUtf8 } from "./runtime.ts";
import { custodyProbe, assertOracleFiles, verifyOracleHashes } from "./oracles.ts";
import { verifyRegistrySource } from "../windows_hover_acceptance.ts";
import {
  readArchive,
  observeInstalledArchive,
  verifyInstalledArchive,
  type InstalledArchive,
} from "./archive.ts";

type Original = {
  schema: string;
  identity: {
    cut: string;
    head: string;
    tag: string;
    sourcePr: string;
    run: string;
    attempt: number;
  };
  policy: {
    sha: string;
    workflowSha256: string;
    consumerSha256: string;
    denoConfigSha256: string;
    custodyProbeSha256: string;
  };
};
const roles = {
  "node22-linux": { runtime: "node", node: "22", platform: "linux" },
  "node24-linux": { runtime: "node", node: "24", platform: "linux" },
  "node24-macos": { runtime: "node", node: "24", platform: "darwin" },
  "node24-windows": { runtime: "node", node: "24", platform: "win32" },
  "deno-linux": { runtime: "deno", node: "24", platform: "linux" },
  "bun-linux": { runtime: "bun", node: "24", platform: "linux" },
} as const;
type Role = keyof typeof roles;
const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const json = (file: string) => JSON.parse(fs.readFileSync(file, "utf8"));
const object = (value: unknown): Record<string, unknown> => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "object required");
  return value as Record<string, unknown>;
};
type Command = {
  label: string;
  command: string;
  args: string[];
  cwd: string;
  began: string;
  ended: string;
  status: number | null;
  signal: string | null;
  error?: string;
  stdout: string;
  stderr: string;
  denoDir: string | null;
  oracleBefore: Record<string, string>;
  oracleAfter?: Record<string, string>;
};
type Installed = {
  native: { path: string; loaded: boolean; sha256: string };
  packages: { name: string; version: string; directory: string; manifestSha256: string }[];
  runtime: { node: string; deno?: { deno: string }; bun?: string; platform: string; arch: string };
};
type Packet = {
  schema: string;
  status: string;
  role: Role;
  identity: Original["identity"];
  policy: Original["policy"];
  consumerSha256: string;
  denoConfigSha256: string;
  effectiveDenoConfigSha256: string;
  run: { id?: string; attempt?: string; sha?: string };
  project: string;
  checkout: string;
  output: string;
  nodeExecutable: string;
  runtime: (typeof roles)[Role];
  commands: Command[];
  packages: Record<string, unknown>[];
  installed?: Installed;
  error?: string;
  jsr?: unknown;
};

export function commandPlan(packet: Packet) {
  const selected = roles[packet.role];
  const platformPath = selected.platform === "win32" ? path.win32 : path.posix;
  const prefix =
    selected.platform === "win32" ? "node_modules/npm/bin" : "../lib/node_modules/npm/bin";
  const npm = platformPath.join(platformPath.dirname(packet.nodeExecutable), prefix, "npm-cli.js");
  const npx = platformPath.join(platformPath.dirname(packet.nodeExecutable), prefix, "npx-cli.js");
  const version = packet.identity.tag.replace(/^v/u, "");
  const root = selected.runtime === "deno" ? `jsr:@vizejs/vize@${version}` : "@vizejs/vize";
  const cache = platformPath.join(packet.project, "deno-cache");
  const probeArgs = [
    "custody.ts",
    root,
    version,
    platformPath.join(packet.output, "installed.json"),
    selected.runtime === "deno" ? platformPath.join(cache, "npm/registry.npmjs.org") : "",
  ];
  const expectedCommands: [string, string, string[]][] = [
    ["node-version", packet.nodeExecutable, ["--version"]],
  ];
  if (selected.runtime === "node")
    expectedCommands.push(
      ["install", packet.nodeExecutable, [npm, "install", "vite@8", "vue@3"]],
      ["jsr-add", packet.nodeExecutable, [npx, "--yes", "jsr", "add", `@vizejs/vize@${version}`]],
      [
        "consumer",
        packet.nodeExecutable,
        ["--experimental-strip-types", "consumer.ts", root, "--vite"],
      ],
      ["custody", packet.nodeExecutable, ["--experimental-strip-types", ...probeArgs]],
    );
  else if (selected.runtime === "bun")
    expectedCommands.push(
      ["runtime-version", "bun", ["--version"]],
      ["install", "bun", ["add", "vite@8", "vue@3"]],
      ["jsr-add", "bunx", ["jsr", "add", `@vizejs/vize@${version}`]],
      ["consumer", "bun", ["run", "consumer.ts", root, "--vite"]],
      ["custody", "bun", ["run", ...probeArgs]],
    );
  else
    expectedCommands.push(
      ["runtime-version", "deno", ["--version"]],
      ["consumer", "deno", ["run", "--config", "deno.json", "--allow-all", "consumer.ts", root]],
      ["custody", "deno", ["run", "--config", "deno.json", "--allow-all", ...probeArgs]],
    );
  return expectedCommands;
}

/** Reject partial packets before the finalizer signs the combined evidence. */
export function verifyConsumer(value: unknown, expected?: unknown) {
  object(value);
  const packet = value as Packet;
  assert.equal(packet.schema, "vize-consumer-recovery-consumer-v1");
  assert.equal(packet.status, "passed");
  assert.ok(Object.hasOwn(roles, packet.role));
  const selected = roles[packet.role];
  assert.deepEqual(packet.runtime, selected);
  if (expected) {
    assert.deepEqual(packet.identity, object(expected).identity);
    assert.deepEqual(packet.policy, object(expected).policy);
  }
  assert.equal(packet.consumerSha256, packet.policy.consumerSha256);
  assert.equal(packet.denoConfigSha256, packet.policy.denoConfigSha256);
  assert.equal(
    packet.consumerSha256,
    "410be5021e5c1bff706191bf68045ddbc2edbaa47b13325ddde3daa22926851c",
  );
  assert.equal(
    packet.denoConfigSha256,
    "7b02ff3799e9e65083d27c1f65260c9a294bb809e79174f828f9bd79148b9700",
  );
  assert.equal(
    packet.policy.workflowSha256,
    "bf5b8d553c05b85afa434a4b4f637a5c5f0b09f9357a6026ab4d0f2014893d81",
  );
  assert.equal(packet.effectiveDenoConfigSha256, digest(effectiveDenoConfig));
  assert.equal(packet.run.sha, packet.policy.sha);
  const platformPath = selected.platform === "win32" ? path.win32 : path.posix;
  assert.ok(platformPath.isAbsolute(packet.project));
  assert.ok(!packet.project.startsWith(packet.checkout + platformPath.sep));
  assert.match(platformPath.basename(packet.nodeExecutable), /^node(?:\.exe)?$/u);
  const version = packet.identity.tag.replace(/^v/u, "");
  const cache = platformPath.join(packet.project, "deno-cache");
  assert.deepEqual(
    packet.commands.map((command) => [command.label, command.command, command.args]),
    commandPlan(packet),
  );
  for (const command of packet.commands) {
    assert.equal(command.status, 0, command.label);
    assert.equal(command.signal, null, command.label);
    assert.equal(typeof command.stdout, "string");
    assert.equal(typeof command.stderr, "string");
    assert.equal(command.cwd, packet.project);
    assert.equal(command.denoDir, selected.runtime === "deno" ? cache : null);
    assert.equal(command.error, undefined);
    verifyOracleHashes(command.oracleBefore, packet.policy, selected.runtime === "deno");
    verifyOracleHashes(command.oracleAfter, packet.policy, selected.runtime === "deno");
  }
  assert.match(packet.commands[0].stdout, new RegExp(`^v${selected.node}\\.`));
  const completed = packet.commands.find((command) => command.label === "consumer");
  assert.ok(completed);
  assert.equal(
    completed.stdout,
    selected.runtime === "deno"
      ? `Verified jsr:@vizejs/vize@${version} (no flags)\n`
      : "Verified @vizejs/vize (--vite)\n",
    "the complete original consumer must reach its exact final witness",
  );
  assert.ok(packet.installed);
  assert.equal(packet.installed.native.loaded, true);
  assert.match(packet.installed.native.sha256, /^[a-f0-9]{64}$/u);
  assert.ok(packet.installed.native.path.endsWith(".node"));
  const actual = packet.installed.runtime;
  assert.equal(actual.platform, selected.platform);
  assert.ok(["x64", "arm64"].includes(actual.arch));
  if (selected.runtime === "deno") assert.equal(actual.deno?.deno, "2.9.7");
  else if (selected.runtime === "bun") assert.equal(actual.bun, "1.3.14");
  else assert.match(actual.node, new RegExp(`^v${selected.node}\\.`));
  const provider = `@vizejs/native-${actual.platform === "win32" ? "win32" : actual.platform}-${actual.arch}${actual.platform === "win32" ? "-msvc" : actual.platform === "linux" ? "-gnu" : ""}`;
  assert.deepEqual(
    packet.packages.map((record) => String(record.name)).sort((a, b) => a.localeCompare(b)),
    [
      "vize",
      "@vizejs/native",
      provider,
      ...(selected.runtime === "deno" ? [] : ["@vizejs/vite-plugin"]),
    ].sort((a, b) => a.localeCompare(b)),
  );
  assert.deepEqual(
    packet.installed.packages.map((record) => record.name).sort((a, b) => a.localeCompare(b)),
    packet.packages.map((record) => String(record.name)).sort((a, b) => a.localeCompare(b)),
    "actual installed package observations must be complete",
  );
  for (const record of packet.packages) {
    const installedObservation: Installed["packages"][number] | undefined =
      packet.installed.packages.find((installed) => installed.name === record.name);
    assert.ok(installedObservation, "actual provider observation required");
    for (const key of ["name", "version", "directory", "manifestSha256"] as const)
      assert.equal(
        record[key],
        installedObservation[key],
        `actual installed ${key} differs from archive observation`,
      );
    verifyInstalledArchive({
      ...record,
      native: record.name === provider ? packet.installed.native : undefined,
    } as InstalledArchive);
    assert.equal(record.version, version);
    assert.equal(record.source, packet.identity.head);
    assert.equal(new URL(String(record.tarball)).origin, "https://registry.npmjs.org");
    assert.equal(record.integrity, `sha512-${String(object(record.archive).sha512)}`);
    assert.match(String(object(record.archive).sha256), /^[a-f0-9]{64}$/u);
    assert.ok(
      Date.parse(String(record.published)) <= Date.parse(packet.commands[0].began) - 86_400_000,
    );
    assert.ok(String(record.directory).startsWith(packet.project + platformPath.sep));
  }
  const nativePackage = packet.packages.find((record) => record.name === provider);
  assert.ok(
    packet.installed.native.path.startsWith(String(nativePackage?.directory) + platformPath.sep),
  );
  assert.ok(packet.jsr);
}

// Supplemental observation runs inside the same runtime and fresh dependency tree.
// The actual acceptance script is copied byte for byte from the frozen release H.

async function main() {
  const [roleValue, outputValue, originalFile] = process.argv.slice(2);
  assert.ok(Object.hasOwn(roles, roleValue), "known consumer role required");
  assert.ok(outputValue && originalFile, "role, output directory and original.json required");
  const role = roleValue as Role;
  const selected = roles[role];
  const original: Original = json(originalFile);
  assert.equal(original.schema, "vize-consumer-recovery-original-v1");
  assert.equal(process.platform, selected.platform);
  assert.equal(process.versions.node.split(".")[0], selected.node);
  assert.equal(process.env.RECOVERY_RUNTIME, selected.runtime);
  assert.equal(process.env.RECOVERY_NODE, selected.node);
  assert.equal(process.env.GITHUB_REF, "refs/heads/main");
  assert.equal(process.env.GITHUB_SHA, original.policy.sha);
  rejectOverrides();
  for (const name of [
    "NODE_PATH",
    "VIZE_BINDING_PATH",
    "NPM_CONFIG_USERCONFIG",
    "DENO_AUTH_TOKENS",
    "DENO_UNSTABLE_BARE_NODE_BUILTINS",
  ]) {
    assert.ok(!process.env[name], `runtime override must be absent: ${name}`);
  }
  const output = path.resolve(outputValue);
  fs.mkdirSync(output, { recursive: true });
  const save = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
  const project = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), `vize-${role}-`)));
  const checkout = fs.realpathSync(process.cwd());
  assert.ok(!project.startsWith(checkout + path.sep), "fresh project must be outside checkout");
  const version = original.identity.tag.replace(/^v/u, "");
  assert.match(version, /^0\.[1-9][0-9]*\.0$/u);
  const source = fs.readFileSync(path.join(path.dirname(originalFile), "consumer.ts"));
  const config = fs.readFileSync(path.join(path.dirname(originalFile), "consumer-deno.json"));
  assert.equal(digest(source), original.policy.consumerSha256);
  assert.equal(digest(config), original.policy.denoConfigSha256);
  const frozen = await fetch(
    `https://raw.githubusercontent.com/ubugeeei-prod/vize/${original.identity.head}/tools/support/release/jsr/consumer.ts`,
    { signal: AbortSignal.timeout(30_000) },
  );
  assert.ok(frozen.ok, `frozen consumer HTTP ${frozen.status}`);
  assert.equal(digest(Buffer.from(await frozen.arrayBuffer())), digest(source));
  fs.writeFileSync(path.join(project, "consumer.ts"), source);
  fs.writeFileSync(
    path.join(project, "package.json"),
    '{ "name": "vize-jsr-consumer", "private": true, "type": "module" }\n',
  );
  fs.writeFileSync(path.join(project, "custody.ts"), custodyProbe);
  const cache = path.join(project, "deno-cache");
  if (selected.runtime === "deno") {
    fs.mkdirSync(cache);
    fs.writeFileSync(path.join(project, "deno.json"), effectiveDenoConfig);
  }
  const environment: NodeJS.ProcessEnv = {
    ...publicEnvironment(process.env),
    npm_config_registry: "https://registry.npmjs.org/",
    ...(selected.runtime === "deno" ? { DENO_DIR: cache } : {}),
  };
  delete environment.GITHUB_TOKEN;
  delete environment.GH_TOKEN;
  const packet: Packet = {
    schema: "vize-consumer-recovery-consumer-v1",
    status: "running",
    role,
    identity: original.identity,
    policy: original.policy,
    consumerSha256: digest(source),
    denoConfigSha256: digest(config),
    effectiveDenoConfigSha256: digest(effectiveDenoConfig),
    run: {
      id: process.env.GITHUB_RUN_ID,
      attempt: process.env.GITHUB_RUN_ATTEMPT,
      sha: process.env.GITHUB_SHA,
    },
    project,
    checkout,
    output,
    nodeExecutable: process.execPath,
    runtime: selected,
    commands: [],
    packages: [],
  };
  save("original.json", original);
  const persist = () => save("consumer.json", packet);
  persist();
  const run = async (label: string, command: string, args: string[]) => {
    const oracleBefore = assertOracleFiles(project, original.policy, selected.runtime === "deno");
    const began = new Date().toISOString();
    const result = await captureCommand(command, args, path.join(output, `${label}-raw`), {
      cwd: project,
      env: environment,
      timeoutMs: 300_000,
    });
    const record = {
      label,
      command,
      args,
      cwd: project,
      began,
      ended: new Date().toISOString(),
      status: result.status,
      signal: result.signal,
      error: result.error,
      stdout: strictUtf8(result.stdout),
      stderr: strictUtf8(result.stderr),
      denoDir: selected.runtime === "deno" ? cache : null,
      oracleBefore,
      oracleAfter: undefined as Record<string, string> | undefined,
    };
    packet.commands.push(record);
    save(`${label}.json`, record);
    persist();
    assert.equal(result.error, undefined, label);
    assert.equal(result.signal, null, label);
    record.oracleAfter = assertOracleFiles(project, original.policy, selected.runtime === "deno");
    save(`${label}.json`, record);
    persist();
    assert.equal(result.status, 0, `${label}: ${record.stderr}`);
    return record.stdout;
  };
  try {
    for (const [label, command, args] of commandPlan(packet)) {
      const stdout = await run(label, command, args);
      if (label === "runtime-version") {
        if (selected.runtime === "bun") assert.equal(stdout.trim(), "1.3.14");
        else assert.match(stdout, /^deno 2\.9\.7\s/u);
      }
    }
    packet.installed = json(path.join(output, "installed.json")) as Installed;
    const reader = publicReader();
    const jsr = await reader.json(`https://jsr.io/@vizejs/vize/${version}_meta.json`);
    const jsrMeta = await reader.json("https://jsr.io/@vizejs/vize/meta.json");
    const jsrPublished = object(object(jsrMeta.versions)[version]).createdAt;
    assert.ok(typeof jsrPublished === "string");
    assert.ok(
      Date.parse(packet.commands[0].began) - Date.parse(jsrPublished) >= 86_400_000,
      "JSR package must be at least 24 hours old",
    );
    const jsrFiles = [];
    for (const [name, value] of Object.entries(object(jsr.manifest))) {
      assert.match(name, /^\/[A-Za-z0-9._/-]+$/u);
      assert.ok(!name.includes(".."));
      const archive = await reader.download(`https://jsr.io/@vizejs/vize/${version}${name}`);
      assert.equal(`sha256-${archive.sha256}`, object(value).checksum);
      assert.equal(archive.bytes, object(value).size);
      jsrFiles.push({ name, ...archive });
    }
    packet.jsr = { version, published: jsrPublished, files: jsrFiles };
    save("jsr-public-identity.json", { metadata: jsr, package: jsrMeta, observed: packet.jsr });
    for (const installed of packet.installed.packages) {
      const name = installed.name;
      const encoded = encodeURIComponent(name);
      const metadata = await reader.json(`https://registry.npmjs.org/${encoded}/${version}`);
      const packument = await reader.json(`https://registry.npmjs.org/${encoded}`);
      const published = object(packument.time)[version];
      assert.ok(typeof published === "string");
      assert.ok(
        Date.now() - Date.parse(published) >= 86_400_000,
        `${name} must be at least 24 hours old`,
      );
      const attestations = await reader.json(
        `https://registry.npmjs.org/-/npm/v1/attestations/${encoded}@${version}`,
      );
      assert.ok(Array.isArray(attestations.attestations));
      const attestation = attestations.attestations
        .map(object)
        .find((value) => value.predicateType === "https://slsa.dev/provenance/v1");
      assert.ok(attestation, `official source observation required: ${name}`);
      const statement = JSON.parse(
        Buffer.from(
          String(object(object(attestation.bundle).dsseEnvelope).payload),
          "base64",
        ).toString("utf8"),
      );
      const dist = object(metadata.dist);
      const archive = await readArchive(String(dist.tarball), String(dist.integrity));
      const identity = verifyRegistrySource(
        name,
        version,
        original.identity.head,
        metadata,
        statement,
      );
      const record = {
        ...identity,
        ...installed,
        published,
        ...archive,
        ...observeInstalledArchive(installed.directory, archive.archiveEntries),
        observation: "registry source metadata; not cryptographic attestation verification",
      };
      packet.packages.push(record);
      save(`package-${name.replaceAll(/[^a-zA-Z0-9.-]/gu, "_")}.json`, {
        metadata,
        published,
        attestations,
        ...archive,
        installed,
      });
      persist();
    }
    packet.status = "passed";
    verifyConsumer(packet, original);
  } catch (error) {
    packet.status = "failed";
    packet.error = error instanceof Error ? error.stack : String(error);
    throw error;
  } finally {
    for (const name of [
      "package.json",
      "package-lock.json",
      "bun.lock",
      "bun.lockb",
      "deno.lock",
      "deno.json",
      "consumer.ts",
      "custody.ts",
    ]) {
      const file = path.join(project, name);
      if (fs.existsSync(file)) fs.copyFileSync(file, path.join(output, name));
    }
    persist();
  }
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await main();
}
