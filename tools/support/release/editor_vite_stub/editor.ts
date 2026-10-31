import assert from "node:assert/strict";
import {
  existsSync,
  lstatSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { InstalledAliasSession } from "../../../../tests/tooling/support/lsp/installed-alias-replay/protocol.ts";
import { asRecord, asString, type JsonRecord } from "../../json-value.ts";
import type { Recorder } from "../../config/windows_consumer/common.ts";
import { inside, inventory } from "./filesystem.ts";
import { hook, observed, verifyJournal } from "./custody.ts";

/** Inspect only this child process's private temporary namespace, never a shared cache. */
export function mirrors(temporary: string): string[] {
  const base = join(temporary, "vize-canon/editor/sessions");
  if (!existsSync(base)) return [];
  const result: string[] = [];
  for (const session of readdirSync(base)
    .filter((name) => name.startsWith("session-"))
    .sort()) {
    const projects = join(base, session, "projects");
    if (!existsSync(projects)) continue;
    for (const project of readdirSync(projects).sort()) {
      const directory = join(projects, project);
      if (lstatSync(directory).isDirectory() && !lstatSync(directory).isSymbolicLink())
        result.push(realpathSync(directory));
    }
  }
  return result;
}
function generatedSources(root: string): { path: string; text: string }[] {
  const result: { path: string; text: string }[] = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    if (entry.name === "node_modules" || entry.isSymbolicLink()) continue;
    const file = join(root, entry.name);
    if (entry.isDirectory()) result.push(...generatedSources(file));
    else if (entry.isFile() && /\.vue\.[cm]?tsx?$/.test(entry.name))
      result.push({ path: file, text: readFileSync(file, "utf8") });
  }
  return result;
}
export function injectLink(mirror: string, temporary: string, modules: string) {
  assert.ok(inside(temporary, mirror));
  assert.equal(realpathSync(mirror), mirror);
  const link = join(mirror, "node_modules");
  assert.ok(
    lstatSync(link).isDirectory() && !lstatSync(link).isSymbolicLink(),
    "only an owned mirror directory may be removed",
  );
  rmSync(link, { recursive: true });
  const kind = process.platform === "win32" ? "junction" : "dir";
  symlinkSync(modules, link, kind);
  assert.ok(lstatSync(link).isSymbolicLink());
  assert.equal(realpathSync(link), modules);
  return {
    path: link,
    kind,
    target: readlinkSync(link),
    resolved: realpathSync(link),
    platform: process.platform,
  };
}
export async function editorAcceptance(
  root: string,
  identity: JsonRecord,
  original: JsonRecord,
  record: Recorder,
): Promise<void> {
  const files = asRecord(original.files),
    modules = realpathSync(join(root, "node_modules"));
  const temporaryPath = join(record.root, "process-temp");
  mkdirSync(temporaryPath);
  const temporary = realpathSync(temporaryPath);
  const tsconfig = {
    compilerOptions: {
      strict: true,
      module: "ESNext",
      moduleResolution: "Bundler",
      target: "ESNext",
      skipLibCheck: true,
      lib: ["ESNext", "DOM"],
    },
    include: ["src/**/*.vue"],
  };
  writeFileSync(join(root, "tsconfig.json"), JSON.stringify(tsconfig, null, 2) + "\n");
  const source = asString(files["src/app.vue"]);
  const probes = ["cold", "warm"].map((name) => ({
    name,
    path: join(root, "src/" + name + ".vue"),
    text: source.replace('"hello"', '"11145-' + name + '"'),
  }));
  for (const probe of probes) writeFileSync(probe.path, probe.text);
  // Keep real pnpm metadata intact. These extra sentinel bytes and link have unique owned paths.
  const sentinel = join(modules, ".11145-retained/pkg");
  mkdirSync(sentinel, { recursive: true });
  writeFileSync(join(sentinel, "index.js"), "export const value = 7;\n");
  symlinkSync(
    sentinel,
    join(modules, "retained-11145"),
    process.platform === "win32" ? "junction" : "dir",
  );
  const before = inventory(modules);
  record.json("editor-external-before.json", before);
  const native = observed(root, identity, record, "editor");
  const session = await InstalledAliasSession.launch({
    nodePath: process.execPath,
    cliPath: asString(identity.cli),
    custodyHookPath: hook,
    projectRoot: root,
    env: { ...native.env, TMP: temporary, TEMP: temporary, TMPDIR: temporary },
    outputRoot: join(record.root, "lsp"),
    timeoutMs: 120_000,
    crossFile: true,
    lint: false,
  });
  const cycles: unknown[] = [];
  try {
    const publication = await session.open(pathToFileURL(join(root, "src/app.vue")).href, source);
    assert.deepEqual(asRecord(publication.params).diagnostics, []);
    const initial = mirrors(temporary);
    assert.equal(
      initial.length,
      1,
      "the actual public editor must create one private project mirror",
    );
    const mirror = initial[0];
    assert.ok(
      generatedSources(mirror).some((item) => item.text.includes('"hello"')),
      "original authored host must be materialized",
    );
    for (const probe of probes) {
      const observed = inventory(modules);
      record.json("editor-" + probe.name + "-before-injection.json", observed);
      assert.deepEqual(observed, before, "external installation changed before injection");
      const injected = injectLink(mirror, temporary, modules);
      const uri = pathToFileURL(probe.path).href;
      const diagnostics = await session.open(uri, probe.text);
      assert.deepEqual(asRecord(diagnostics.params).diagnostics, []);
      const reply = await session.request("textDocument/definition", {
        textDocument: { uri },
        position: { line: 5, character: 9 },
      });
      // The request forces an actual semantic round trip; complete RPC is retained without
      // claiming general navigation conformance from this filesystem acceptance case.
      assert.ok(Object.hasOwn(reply, "result"));
      assert.deepEqual(
        mirrors(temporary),
        initial,
        "same live project must recover its own directory",
      );
      const link = join(mirror, "node_modules"),
        metadata = lstatSync(link);
      assert.ok(
        metadata.isDirectory() && !metadata.isSymbolicLink(),
        "stale whole-install link must be detached before runtime writes",
      );
      assert.equal(realpathSync(link), link);
      const generated = generatedSources(mirror).filter((item) =>
        item.text.includes('"11145-' + probe.name + '"'),
      );
      assert.ok(generated.length > 0, "fresh host must reach actual mirror materialization");
      const after = inventory(modules);
      record.json("editor-" + probe.name + "-external-after.json", after);
      assert.deepEqual(after, before, "public editor changed an external file, mode, or link");
      cycles.push({
        probe,
        injected,
        mirror,
        generated,
        diagnostics,
        reply,
        virtualModules: inventory(link),
      });
    }
  } finally {
    const receipt = await session.shutdown();
    record.json("editor-session.json", receipt);
    record.json("editor-external-after-shutdown.json", inventory(modules));
    assert.equal(receipt.closed, true);
    assert.equal(receipt.exitCode, 0);
    assert.equal(receipt.signal, null);
    assert.equal(receipt.remainingFrameBytes, 0);
    assert.equal(receipt.parsingFailed, false);
    assert.deepEqual(receipt.failures, []);
  }
  verifyJournal(native.journal, identity, 0);
  assert.deepEqual(
    inventory(modules),
    before,
    "shutdown must preserve the whole external installation",
  );
  for (const name of ["vite.config.ts", "src/app.vue"])
    assert.equal(readFileSync(join(root, name), "utf8"), files[name]);
  record.json("editor-acceptance.json", {
    platform: process.platform,
    cycles,
    nativeJournal: native.journal,
    scenario:
      "Deliberate stale mirror whole-link injection; not a claim that the minimal fresh pnpm workspace naturally creates that topology",
    assertion:
      "actual public LSP cold and warm new-host preparation detaches the whole symlink/junction and leaves the complete real installation inventory identical",
  });
}
