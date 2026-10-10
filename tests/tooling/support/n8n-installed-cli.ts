import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { performance } from "node:perf_hooks";
import {
  installedAuthority,
  exactPath,
  sha256,
  type InstalledCampaignPlan,
} from "./n8n-installed-authority.ts";
import { root, projection } from "./n8n-cli-config-inputs.mjs";
import { errorPacket } from "./n8n-cli-config-oracle.mjs";
import { recordCliPackets, writeJson } from "./n8n-cli-config-workspace.mjs";
import type { CliPacket } from "./n8n-installed-contract-types.ts";
import { rejectOverrides } from "../../../tools/support/release/public_acceptance/installed.ts";

/** Refuse newly introduced ambient preloads before any invocation payload checks or process startup. */
export function installedInvocationPreflight(checkPayload: () => void): void {
  rejectOverrides();
  checkPayload();
}

export interface CliInvocation {
  binary: string;
  workspace: string;
  packageRoot: string;
  files: string[];
  config: { path: string; [key: string]: unknown };
  receiptPath: string;
  outsideCwd?: boolean;
}
interface JournalEvent {
  schema: string;
  pid: number;
  event: string;
  installRoot?: string;
  nativePath?: string;
  sha256?: string;
  node?: string;
  argv?: string[];
  actualPath?: string;
  expectedNative?: boolean;
  corsaPath?: string;
  code?: number;
}
export function validateNativeJournal(
  raw: string,
  expected: {
    installRoot: string;
    nativePath: string;
    nativeSha256: string;
    node: string;
    argv: string[];
    pid: number;
    status: number;
    corsaPath: string;
  },
): void {
  assert.ok(raw.length > 0 && raw.endsWith("\n"), "complete native journal required");
  const events = raw
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line) as JournalEvent);
  const first = events[0];
  assert.equal(first.event, "initialized");
  assert.equal(first.installRoot, expected.installRoot);
  assert.equal(first.nativePath, expected.nativePath);
  assert.equal(first.sha256, expected.nativeSha256);
  assert.equal(first.node, expected.node);
  assert.deepEqual(first.argv, expected.argv);
  assert.equal(events.filter(({ event }) => event === "initialized").length, 1);
  let attempts = 0;
  let returned = 0;
  for (const event of events) {
    assert.equal(event.schema, "vize-public-native-custody-event-v1");
    assert.equal(event.pid, expected.pid);
    assert.ok(["initialized", "attempt", "returned", "exit"].includes(event.event));
    if (event.expectedNative === true) {
      assert.equal(event.actualPath, expected.nativePath);
      assert.equal(event.sha256, expected.nativeSha256);
      if (event.event === "attempt") attempts++;
      if (event.event === "returned") {
        assert.equal(attempts, 1);
        assert.equal(returned, 0);
        assert.equal(event.corsaPath, expected.corsaPath);
        returned++;
      }
    }
  }
  assert.equal(attempts, 1);
  assert.equal(returned, 1, "actual native loader must return successfully");
  assert.equal(events.filter(({ event }) => event === "exit").length, 1);
  assert.equal(events.at(-1)?.event, "exit");
  assert.equal(events.at(-1)?.code, expected.status);
}

export function installedCliRuntime(output: string, planPath: string) {
  assert.ok(planPath, "an explicit reviewed installed-campaign plan is required");
  const plan = JSON.parse(fs.readFileSync(exactPath(planPath), "utf8")) as InstalledCampaignPlan;
  const custody = installedAuthority(plan, root);
  const { authority } = custody;
  assert.ok(path.isAbsolute(output));
  exactPath(path.dirname(output), undefined, true);
  assert.equal(fs.existsSync(output), false, "installed campaign output must be new");
  assert.equal(output.startsWith(authority.installRoot + path.sep), false);
  fs.mkdirSync(output);
  const campaign = fs.mkdtempSync(path.join(authority.installRoot, ".vize-n8n-campaign-"));
  const configEntry = exactPath(
    createRequire(authority.cli.binPath).resolve("vize/config"),
    path.join(authority.installRoot, "node_modules/vize"),
  );
  const identity = {
    kind: "public-npm-install" as const,
    source: plan.source,
    receiptPath: custody.receiptPath,
    receiptSha256: custody.receiptSha256,
    installRoot: authority.installRoot,
    cli: authority.cli,
    configEntry,
    configSha256: sha256(fs.readFileSync(configEntry)),
    native: authority.native,
    collectorAuthority: authority.collectorAuthority,
    qualification:
      "reviewed installed custody; no workspace build identity or complete adoption claim",
  };
  writeJson(path.join(output, "producer.json"), { identity, authority });
  const consumedFiles = new Map<string, string>();
  const pinFile = (filename: string, expectedDigest?: string) => {
    const canonical = exactPath(filename, campaign);
    const digest = sha256(fs.readFileSync(canonical));
    if (expectedDigest !== undefined) assert.equal(digest, expectedDigest);
    if (consumedFiles.has(canonical)) assert.equal(digest, consumedFiles.get(canonical));
    consumedFiles.set(canonical, digest);
  };
  const recheckInvocation = () =>
    installedInvocationPreflight(() => {
      for (const [filename, digest] of [
        [authority.cli.binPath, authority.cli.binSha256],
        [authority.cli.distCliPath, authority.cli.distCliSha256],
        [authority.node.path, authority.node.sha256],
        [authority.custodyHook.path, authority.custodyHook.sha256],
        [configEntry, identity.configSha256],
      ])
        assert.equal(sha256(fs.readFileSync(exactPath(filename))), digest);
    });
  const workspace = (name: string) => {
    assert.match(name, /^[a-z-]+$/u);
    return path.join(campaign, name);
  };
  const installConfigPackages = (destination: string) => {
    assert.ok(destination.startsWith(campaign + path.sep));
    fs.mkdirSync(path.join(destination, ".git"), { recursive: true });
    fs.writeFileSync(path.join(destination, "package.json"), '{"private":true,"type":"module"}\n');
    const settings = path.join(destination, "node_modules/@vize-acceptance/n8n-cli-settings");
    fs.mkdirSync(settings, { recursive: true });
    fs.writeFileSync(
      path.join(settings, "package.json"),
      JSON.stringify({
        name: "@vize-acceptance/n8n-cli-settings",
        private: true,
        type: "module",
        exports: { ".": "./index.js" },
      }) + "\n",
    );
    fs.writeFileSync(
      path.join(settings, "index.js"),
      "export const settings = " + JSON.stringify({ linter: projection.linter }, null, 2) + ";\n",
    );
    assert.equal(
      createRequire(path.join(destination, "package.json")).resolve("vize/config"),
      configEntry,
    );
    return [
      "package.json",
      "node_modules/@vize-acceptance/n8n-cli-settings/package.json",
      "node_modules/@vize-acceptance/n8n-cli-settings/index.js",
    ].map((file) => {
      const filename = path.join(destination, file);
      pinFile(filename);
      return { file, sha256: consumedFiles.get(filename)! };
    });
  };
  const runCli = ({
    binary,
    workspace: directory,
    packageRoot,
    files,
    config,
    receiptPath,
    outsideCwd = false,
  }: CliInvocation): CliPacket[] => {
    recheckInvocation();
    assert.equal(binary, authority.cli.binPath);
    assert.ok(directory.startsWith(campaign + path.sep));
    assert.ok(receiptPath.startsWith(output + path.sep));
    assert.equal(createRequire(config.path).resolve("vize/config"), configEntry);
    const cwd = outsideCwd ? directory : path.join(directory, packageRoot);
    const patterns = outsideCwd ? files.map((file) => path.join(packageRoot, file)) : files;
    assert.equal(typeof config.sha256, "string", "complete authored config digest required");
    pinFile(config.path, config.sha256 as string);
    for (const file of patterns) pinFile(path.join(cwd, file));
    const args = [
      "lint",
      "--config",
      config.path,
      "--format",
      "json",
      "--locale",
      "en",
      "--help-level",
      "none",
      ...patterns,
    ];
    fs.mkdirSync(path.dirname(receiptPath), { recursive: true });
    const journalPath = receiptPath.replace(/\.json$/u, ".native-journal.jsonl");
    assert.equal(fs.existsSync(journalPath), false);
    const observer = {
      schema: "vize-public-native-custody-v1",
      installRoot: authority.installRoot,
      nativePath: authority.native.path,
      nativeSha256: authority.native.sha256,
      journalPath,
    };
    const argv = ["--require", authority.custodyHook.path, binary, ...args];
    const started = performance.now();
    const result = spawnSync(authority.node.path, argv, {
      cwd,
      encoding: "utf8",
      timeout: 120_000,
      maxBuffer: 64 * 1024 * 1024,
      env: { ...process.env, NO_COLOR: "1", VIZE_PUBLIC_NATIVE_CUSTODY: JSON.stringify(observer) },
    });
    const journal = fs.existsSync(journalPath) ? fs.readFileSync(journalPath, "utf8") : "";
    writeJson(receiptPath, {
      binary,
      args,
      cwd,
      config,
      command: authority.node.path,
      argv,
      elapsedMs: performance.now() - started,
      pid: result.pid,
      status: result.status,
      signal: result.signal,
      ...(result.error ? { error: errorPacket(result.error) } : {}),
      stdout: result.stdout,
      stderr: result.stderr,
      nativeJournal: { path: journalPath, sha256: sha256(journal), raw: journal },
    });
    assert.equal(result.error, undefined, receiptPath);
    assert.equal(result.signal, null, receiptPath);
    assert.ok(result.status === 0 || result.status === 1, receiptPath);
    validateNativeJournal(journal, {
      ...observer,
      node: authority.node.path,
      argv: [authority.node.path, binary, ...args],
      pid: result.pid,
      status: result.status,
      corsaPath: authority.bundledCorsa.path,
    });
    assert.equal(sha256(fs.readFileSync(authority.native.path)), authority.native.sha256);
    assert.equal(sha256(fs.readFileSync(configEntry)), identity.configSha256);
    recheckInvocation();
    const packets = recordCliPackets(JSON.parse(result.stdout)) as CliPacket[];
    assert.deepEqual(
      packets.map(({ file }) => file),
      [...patterns].sort((a, b) => Buffer.compare(Buffer.from(a), Buffer.from(b))),
    );
    const errors = packets.reduce((sum, packet) => sum + packet.errorCount, 0);
    assert.equal(result.status, errors ? 1 : 0);
    writeJson(receiptPath.replace(/\.json$/u, ".recorded.json"), packets);
    return packets;
  };
  const recheck = () => {
    custody.recheck();
    for (const [filename, digest] of consumedFiles)
      assert.equal(sha256(fs.readFileSync(exactPath(filename, campaign))), digest);
  };
  return {
    identity,
    binary: authority.cli.binPath,
    workspace,
    installConfigPackages,
    runCli,
    recheck,
  };
}
