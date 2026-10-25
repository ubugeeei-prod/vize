import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";

const root = process.cwd();
const load = (name) => import(pathToFileURL(path.join(root, name)).href);
const patch = await load("tests/_helpers/realworld-patch.ts");
const check = await load("tests/_helpers/realworld-typecheck.ts");
const { compatProbes } = await load("tests/_helpers/compat-ratchet.ts");
const { resolveVueTscManifestPath } = await load("tests/_helpers/vue-tsc-manifest.ts");
const { compareTypecheckDiagnostics } = await load("tools/support/compat/fixtures/typecheck-divergence.mjs");
const { LspSession } = await load("tests/tooling/support/lsp/session.ts");
const output = path.join(root, "target/context-packet-evidence");
fs.mkdirSync(output, { recursive: true });
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const save = (name, value) => fs.writeFileSync(path.join(output, name), JSON.stringify(value, null, 2) + "\n");
const identify = (binary) => ({
  binary,
  realpath: fs.realpathSync(binary),
  sha256: sha256(fs.readFileSync(fs.realpathSync(binary))),
  version: execFileSync(binary, ["--version"], { encoding: "utf8" }).trim(),
});
const corsa = check.resolveTsgoBinary();
const vueTsc = check.resolveVueTscBinary();
const vueTscManifest = resolveVueTscManifestPath(vueTsc);
assert.ok(vueTscManifest);
const sourceRevision = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
assert.equal(sourceRevision, process.env.SOURCE_REVISION);
save("providers.json", {
  sourceRevision,
  evidenceRevision: process.env.GITHUB_SHA,
  sourceBuild: JSON.parse(fs.readFileSync(path.join(root, "target/ci/vize.differential-build.json"), "utf8")),
  vize: identify(process.env.VIZE_TEST_BIN),
  nativeTypeScript: identify(corsa),
  reference: { ...identify(vueTsc), manifest: JSON.parse(fs.readFileSync(vueTscManifest, "utf8")), manifestSha256: sha256(fs.readFileSync(vueTscManifest)) },
});

const probe = compatProbes.find((entry) => entry.fixtureId === "pinia");
assert.ok(probe);
await patch.withPinnedFixtureWorkspace({ fixtureId: probe.fixtureId, includePaths: probe.includePaths }, async (fixture) => {
  check.symlinkVueTypes(fixture.workspaceDir);
  const config = {
    compilerOptions: { lib: ["ES2022", "DOM", "DOM.Iterable"], module: "ESNext", moduleResolution: "bundler", noEmit: true, skipLibCheck: true, strict: true, target: "ES2022", types: [] },
    include: probe.include,
  };
  fixture.write("tsconfig.json", JSON.stringify(config, null, 2) + "\n");
  const sources = probe.includePaths.map((file) => ({ file, source: fixture.read(file), sha256: sha256(fixture.read(file)) }));
  const vize = check.runVizeCheck(fixture.workspaceDir, corsa, []);
  const reference = check.runVueTsc(fixture.workspaceDir, vueTsc);
  const comparison = compareTypecheckDiagnostics({ projectId: "pinia", cwd: fixture.workspaceDir, vizeReport: vize.report, vueTscOutput: `${reference.stdout}\n${reference.stderr}` });
  save("pinia.json", { revision: fixture.entry.revision, workspaceDir: fixture.workspaceDir, config, sources, vize, reference, comparison });
  assert.equal(comparison.summary.vizeDiagnosticCount, 7);
  assert.equal(comparison.summary.baselineDiagnosticCount, 7);
  assert.equal(comparison.summary.sharedCount, 6);
  assert.equal(comparison.summary.messageMismatchCount, 1);
  assert.equal(comparison.summary.falsePositiveCount, 0);
  assert.equal(comparison.summary.falseNegativeCount, 0);
});

const file = "src/views/dashboard/admin/components/TransactionTable.vue";
const canonical = "Property 'list' does not exist on the component instance.";
const templateDiagnostic = { range: { start: { line: 1, character: 19 }, end: { line: 1, character: 23 } }, severity: 1, code: 2339, source: "vize/types", message: canonical };
const memberDiagnostic = { range: { start: { line: 49, character: 13 }, end: { line: 49, character: 17 } }, severity: 1, code: 2339, source: "vize/types", message: "Property 'list' does not exist on type '__VizeThis'." };
const variants = [
  { id: "legacy-rename", dialect: { compiler: { compatibility: { vueVersion: "2" } } }, legacy: true, method: true },
  { id: "unified-rename", dialect: { vue: { version: "2.7" } }, legacy: true, method: true },
  { id: "unified-alone-rename", dialect: { vue: { version: "2.7" } }, legacy: false, method: true },
  { id: "legacy-data-key", dialect: { compiler: { compatibility: { vueVersion: "2" } } }, legacy: true, method: false },
];
for (const variant of variants) {
  await patch.withPinnedFixtureWorkspace({ fixtureId: "vue-element-admin", includePaths: [file] }, async (fixture) => {
    const packet = { variant: variant.id, revision: fixture.entry.revision, phases: [] };
    check.symlinkVueTypes(fixture.workspaceDir);
    fixture.write("src/api/remote-search.d.ts", variant.method
      ? 'export function transactionList(): Promise<{ data: { items: Array<{ order_no: string; price: number; status: "success" | "pending" }> } }>;\n'
      : 'export declare function searchUser(name: string): Promise<{ data: { items: unknown[] } }>;\nexport declare function transactionList(): Promise<{ data: { items: unknown[] } }>;\n');
    const compilerOptions = variant.method
      ? { allowJs: true, checkJs: true, lib: ["ES2022", "DOM"], paths: { "@/*": ["./src/*"] }, skipLibCheck: true, strict: false }
      : { target: "esnext", module: "esnext", moduleResolution: "bundler", allowJs: true, checkJs: true, strict: false, skipLibCheck: true, noEmit: true, paths: { "@/*": ["./src/*"] } };
    const config = { ...variant.dialect, globalTypes: { toThousandFilter: "any" }, typeChecker: { corsaPath: corsa, ...(variant.legacy ? { legacyVue2: true } : {}) } };
    fixture.write("tsconfig.json", JSON.stringify({ compilerOptions, include: [variant.method ? "src" : "src/**/*"] }, null, 2) + "\n");
    fixture.writeVueConfig(JSON.stringify(config, null, 2) + "\n");
    const source = fixture.read(file);
    const uri = pathToFileURL(fixture.resolve(file)).href;
    const session = new LspSession();
    const originalNotify = session.notify.bind(session);
    const notifications = [];
    session.notify = (method, params) => { notifications.push({ method, params }); return originalNotify(method, params); };
    const publications = [];
    session.notificationObservers.push((method, params) => publications.push({ method, params }));
    try {
      packet.source = source;
      packet.sourceSha256 = sha256(source);
      packet.configuration = config;
      packet.initialization = await session.initialize(fixture.workspaceDir, { editor: true, ...(variant.legacy ? { legacyVue2: true } : {}), lint: false, typecheck: true });
      const pid = session.processId;
      for (const [phase, version] of [["clean", 1], ["broken", 2], ["restored", 3]]) {
        let text = source;
        if (phase === "broken") {
          text = fixture.applyExactPatch(file, "list: null", "tableList: null");
          if (variant.method) text = fixture.applyExactPatch(file, "this.list = ", "this.tableList = ");
        } else if (phase === "restored") {
          text = fixture.applyExactPatch(file, "tableList: null", "list: null");
          if (variant.method) text = fixture.applyExactPatch(file, "this.tableList = ", "this.list = ");
          assert.equal(text, source);
        }
        const cli = check.runVizeCheck(fixture.workspaceDir, corsa, [file]);
        if (version === 1) session.notify("textDocument/didOpen", { textDocument: { uri, languageId: "vue", version, text } });
        else session.notify("textDocument/didChange", { textDocument: { uri, version }, contentChanges: [{ text }] });
        const publication = await session.waitForNotification("textDocument/publishDiagnostics", (params) => params.uri === uri && params.version === version, 120_000);
        packet.phases.push({ phase, version, source: text, cli, publication });
        save(`vue2-${variant.id}.json`, { ...packet, notifications, publications });
        const expected = phase === "broken" ? (variant.method ? [templateDiagnostic] : [templateDiagnostic, memberDiagnostic]) : [];
        const ordered = [...publication.diagnostics].sort((a, b) => a.range.start.line - b.range.start.line || a.range.start.character - b.range.start.character);
        assert.deepEqual(ordered, expected);
        assert.equal(cli.status, phase === "broken" ? 1 : 0);
        assert.equal(cli.report.errorCount, expected.length);
        assert.equal(cli.report.warningCount, 0);
        assert.equal(cli.report.fileCount, 1);
        assert.equal(session.processId, pid);
      }
    } finally {
      await session.shutdown();
      packet.stderr = session.stderrText;
      save(`vue2-${variant.id}.json`, { ...packet, notifications, publications });
    }
  });
}
save("completed.json", { sourceRevision, piniaPairs: 7, vue2Variants: variants.length, vue2Phases: variants.length * 3 });
