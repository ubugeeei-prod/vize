import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  htmlInstalledAuthority,
  installedHtmlPreflight,
  campaignAuthority,
  sourceRoot,
} from "./oxlint-installed-html-authority.ts";
import { retainHtmlProcess } from "./oxlint-installed-html-packets.ts";
import { verifyStockArchives } from "./oxlint-installed-html-stock.ts";
import { installedBarePlugin } from "./oxlint-installed-html-plugin.ts";
import { exactPath, sha256 } from "./n8n-installed-authority.ts";
import { validateNativeJournal } from "./n8n-installed-cli.ts";
import { snapshot } from "../../../npm/oxlint/src/test-support/html-cli-oracles.mjs";
import { presentationInputs } from "../../../npm/oxlint/src/cli/presentation-context.ts";
import type { HtmlCampaignPlan, HtmlCapture, HtmlProcess } from "./oxlint-installed-html-types.ts";

function files(directory: string): Array<{ path: string; bytes: number; sha256: string }> {
  const result = [];
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    assert.equal(
      entry.isSymbolicLink(),
      false,
      "provider payload cannot redirect through symlinks",
    );
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) result.push(...files(filename));
    else {
      assert.equal(entry.isFile(), true);
      const bytes = fs.readFileSync(exactPath(filename));
      result.push({ path: filename, bytes: bytes.length, sha256: sha256(bytes) });
    }
  }
  return result.sort((a, b) => (a.path < b.path ? -1 : a.path > b.path ? 1 : 0));
}

export function pinnedInstalledProvider(
  plan: HtmlCampaignPlan["providers"][number],
  output: string,
) {
  const root = exactPath(plan.installRoot, undefined, true);
  assert.equal(
    root.startsWith(sourceRoot + path.sep),
    false,
    "pinned host must be independently installed",
  );
  const lockPath = exactPath(path.join(root, "package-lock.json"), root);
  const lockBytes = fs.readFileSync(lockPath);
  assert.equal(sha256(lockBytes), plan.packageLockSha256);
  const lock = JSON.parse(lockBytes.toString("utf8"));
  assert.equal(lock.lockfileVersion, 3);
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "package.json"), "utf8"));
  assert.equal(manifest.dependencies.oxlint, plan.version);
  const packages: Array<{
    version: string;
    link?: boolean;
    optional?: boolean;
    resolved: string;
    integrity: string;
    name: string;
    directory: string;
    manifestBytes: number[];
    files: ReturnType<typeof files>;
  }> = [];
  for (const [location, value] of Object.entries(lock.packages)) {
    if (location !== "node_modules/oxlint" && !location.startsWith("node_modules/@oxlint/binding-"))
      continue;
    const entry = value as {
      version: string;
      link?: boolean;
      optional?: boolean;
      resolved: string;
      integrity: string;
    };
    assert.equal(entry.version, plan.version);
    assert.equal(entry.link, undefined);
    assert.equal(new URL(entry.resolved).origin, "https://registry.npmjs.org");
    assert.match(entry.integrity, /^sha512-[A-Za-z0-9+/]{86}==$/u);
    const directory = path.join(root, location);
    if (!fs.existsSync(directory)) {
      assert.equal(entry.optional, true);
      continue;
    }
    exactPath(directory, root, true);
    const bytes = fs.readFileSync(path.join(directory, "package.json"));
    const actual = JSON.parse(bytes.toString("utf8"));
    assert.equal(actual.name, location.slice("node_modules/".length));
    assert.equal(actual.version, plan.version);
    packages.push({
      ...entry,
      name: actual.name,
      directory,
      manifestBytes: Array.from(bytes),
      files: files(directory),
    });
  }
  assert.ok(packages.some(({ name }) => name === "@oxlint/binding-darwin-arm64"));
  const oxlint = packages.find(({ name }) => name === "oxlint");
  assert.ok(oxlint);
  const engineManifest = JSON.parse(Buffer.from(oxlint.manifestBytes).toString("utf8"));
  const engine = exactPath(
    path.join(oxlint.directory, engineManifest.bin.oxlint),
    oxlint.directory,
  );
  const identity = {
    ...plan,
    lockPath,
    lockBytes: Array.from(lockBytes),
    packages,
    engine,
    engineSha256: sha256(fs.readFileSync(engine)),
    publicArchives: verifyStockArchives(
      packages,
      path.join(output, "stock-public-archive-" + plan.version + ".json"),
    ),
  };
  const recheck = () => {
    assert.equal(sha256(fs.readFileSync(lockPath)), plan.packageLockSha256);
    for (const item of packages) assert.deepEqual(files(item.directory), item.files);
  };
  recheck();
  return { identity, engine, recheck };
}

function optionalNativeJournal(
  raw: string,
  expected: Parameters<typeof validateNativeJournal>[1],
  required: boolean,
) {
  if (required) return validateNativeJournal(raw, expected);
  assert.ok(raw.endsWith("\n"));
  const events = raw
    .trimEnd()
    .split("\n")
    .map((line) => JSON.parse(line));
  assert.deepEqual(
    events.map(({ event }) => event),
    ["initialized", "exit"],
  );
  assert.equal(events[0].schema, "vize-public-native-custody-event-v1");
  assert.equal(events[0].pid, expected.pid);
  assert.equal(events[0].nativePath, expected.nativePath);
  assert.equal(events[0].sha256, expected.nativeSha256);
  assert.equal(events[0].installRoot, expected.installRoot);
  assert.equal(events[0].node, expected.node);
  assert.deepEqual(events[0].argv, expected.argv);
  assert.equal(events[1].pid, expected.pid);
  assert.equal(events[1].schema, events[0].schema);
  assert.equal(events[1].code, expected.status);
}

/** No workspace dist, binary path substitution, or ambient preload is admitted. */
export function installedHtmlRuntime(output: string, planPath: string) {
  installedHtmlPreflight();
  assert.ok(planPath, "an explicit reviewed installed HTML campaign plan is required");
  const planBytes = fs.readFileSync(exactPath(planPath));
  const plan = JSON.parse(planBytes.toString("utf8")) as HtmlCampaignPlan;
  const custody = htmlInstalledAuthority(plan);
  const { authority } = custody;
  assert.equal(
    fs.realpathSync(process.execPath),
    authority.node.path,
    "campaign driver must use the recorded collector Node",
  );
  assert.equal(process.version, authority.node.version);
  assert.equal(sha256(fs.readFileSync(authority.node.path)), authority.node.sha256);
  assert.ok(path.isAbsolute(output));
  exactPath(path.dirname(output), undefined, true);
  assert.equal(fs.existsSync(output), false, "campaign output must be new");
  assert.equal(output.startsWith(authority.installRoot + path.sep), false);
  fs.mkdirSync(output);
  const workspace = fs.mkdtempSync(path.join(authority.installRoot, ".vize-original-html-"));
  const observer = exactPath(
    path.join(sourceRoot, "tests/tooling/support/oxlint-installed-html-observer.cjs"),
    sourceRoot,
  );
  const observerSha256 = sha256(fs.readFileSync(observer));
  const pluginEntry = exactPath(
    path.join(custody.plugin.packageDirectory, "dist/index.mjs"),
    custody.plugin.packageDirectory,
  );
  const identity = {
    kind: "public-npm-install",
    source: plan.source,
    includedRoutes: plan.includedRoutes,
    routeProofs: custody.routeProofs,
    publicationProofs: custody.publicationProofs,
    planPath,
    planSha256: sha256(planBytes),
    receiptPath: custody.receiptPath,
    receiptSha256: custody.receiptSha256,
    authority,
    collectorAuthority: authority.collectorAuthority,
    campaignAuthority: custody.producer,
    plugin: custody.plugin,
    binary: custody.binary,
    binarySha256: sha256(fs.readFileSync(custody.binary)),
    dist: custody.dist,
    distSha256: sha256(fs.readFileSync(custody.dist)),
    pluginEntry,
    native: authority.native,
    observer,
    observerSha256,
    qualification:
      "prepared bounded original HTML public CLI campaign; no complete adoption or performance claim",
  };
  fs.writeFileSync(path.join(output, "producer.json"), JSON.stringify(identity, null, 2) + "\n");
  const recheck = () => {
    installedHtmlPreflight();
    assert.equal(sha256(fs.readFileSync(planPath)), identity.planSha256);
    assert.deepEqual(campaignAuthority(), custody.producer);
    for (const file of authority.collectorAuthority.files)
      assert.equal(
        sha256(fs.readFileSync(exactPath(path.join(sourceRoot, file.path), sourceRoot))),
        file.sha256,
      );
    custody.recheck();
  };
  const run = (
    capture: HtmlCapture,
    save: () => void,
    provider: ReturnType<typeof pinnedInstalledProvider>,
    name: string,
    format: string,
    root: string,
    temporary: string,
    args: string[],
    environment: NodeJS.ProcessEnv,
    publicCli: boolean,
    nativeRequired: boolean,
  ): HtmlProcess => {
    recheck();
    provider.recheck();
    assert.ok(root.startsWith(workspace + path.sep));
    installedBarePlugin(root, authority.installRoot, custody.plugin.packageDirectory, pluginEntry);
    const id = `${provider.identity.version}-${name}-${format}-${publicCli ? "public" : "stock"}`;
    const journalPath = path.join(output, id + ".native.jsonl");
    const eventsPath = path.join(output, id + ".events.jsonl");
    const nativeObserver = {
      schema: "vize-public-native-custody-v1",
      installRoot: authority.installRoot,
      nativePath: authority.native.path,
      nativeSha256: authority.native.sha256,
      journalPath,
    };
    const htmlObserver = {
      schema: "vize.oxlint.public-html-observer-v1",
      installRoot: authority.installRoot,
      nativePath: authority.native.path,
      nativeSha256: authority.native.sha256,
      eventsPath,
      observerSha256,
      source: plan.source,
    };
    const entrypoint = publicCli ? custody.binary : provider.engine;
    const argv = publicCli
      ? ["--require", authority.custodyHook.path, "--require", observer, entrypoint, ...args]
      : [entrypoint, ...args];
    const before = snapshot(path.dirname(root));
    const result = spawnSync(authority.node.path, argv, {
      cwd: root,
      env: {
        ...environment,
        TMPDIR: temporary,
        ...(publicCli
          ? {
              VIZE_PUBLIC_NATIVE_CUSTODY: JSON.stringify(nativeObserver),
              VIZE_OXLINT_PUBLIC_CUSTODY: JSON.stringify(htmlObserver),
            }
          : {}),
      },
      timeout: 120_000,
      maxBuffer: 64 * 1024 * 1024,
    });
    const { events, rawJournal, stdout, stderr } = retainHtmlProcess(
      capture,
      save,
      {
        fixture: name,
        format,
        publicCli,
        cwd: root,
        command: authority.node.path,
        argv,
        entrypoint,
        args,
        provider: provider.identity,
        before,
        environment: Object.fromEntries(
          presentationInputs.map((key) => [key, environment[key] ?? null]),
        ),
      },
      result,
      publicCli ? eventsPath : undefined,
      publicCli ? journalPath : undefined,
    );
    const after = snapshot(path.dirname(root));
    capture.observations.push({ kind: "custody", fixture: name, format, after });
    save();
    assert.equal(result.signal, null);
    assert.equal(result.error, undefined);
    assert.deepEqual(after, before);
    if (publicCli) {
      assert.ok(result.status === 0 || result.status === 1);
      optionalNativeJournal(
        rawJournal,
        {
          ...nativeObserver,
          node: authority.node.path,
          argv: [authority.node.path, entrypoint, ...args],
          pid: result.pid,
          status: result.status,
          corsaPath: "",
        },
        nativeRequired,
      );
      assert.deepEqual(
        events.filter(({ kind }) => kind === "observer-initialized").map(({ pid }) => pid),
        [result.pid],
      );
      assert.equal(events.at(-1)?.kind, "observer-exit");
      for (const event of events) {
        assert.equal(event.pid, result.pid);
        assert.equal(event.binary, authority.native.path);
        assert.equal(event.binarySha256, authority.native.sha256);
        assert.equal(event.observerSha256, observerSha256);
        assert.deepEqual(event.source, plan.source);
      }
    }
    recheck();
    provider.recheck();
    return {
      ...result,
      stdout,
      stderr,
      events,
    };
  };
  return { identity, workspace, run, recheck, plan };
}
