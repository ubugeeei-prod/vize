import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { spawnSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import {
  originalControlPins,
  currentControlPins,
  commands,
  benchmarkNames,
  movedTargets,
  canonicalPatchProfile,
} from "./control-contract.ts";

export async function createCampaignPlan(root: string) {
  const sha256 = (b: Buffer) => crypto.createHash("sha256").update(b).digest("hex");
  const git = (...argv: string[]) => {
    const r = spawnSync("git", ["--no-replace-objects", ...argv], { cwd: root, encoding: "utf8" });
    assert.equal(r.error, undefined);
    assert.equal(r.status, 0, r.stderr);
    return r.stdout.trimEnd();
  };
  const read = (file: string) => fs.readFileSync(path.join(root, file));
  const fromRepo = (file: string) => import(pathToFileURL(path.join(root, file)).href);
  const { loadFormatterHistoryAudit, validateFormatterHistoryExecution } = await fromRepo(
    "tests/differential/formatter-history-audit.ts",
  );
  const { loadFormatterApiManifest, validateFormatterApiReport } = await fromRepo(
    "tests/differential/formatter-api.mjs",
  );
  const { writeBuildReceipt, validateBuildReceipt, expectedBuildIdentity } = await fromRepo(
    "tests/differential/build-receipt.mjs",
  );
  const { audit, cases } = loadFormatterHistoryAudit(root);
  const { retainedFormatterFunction } = await fromRepo(
    "tests/differential/formatter-history-current-witness.ts",
  );
  assert.equal(cases.size, 300);
  const source = {
    revision: git("rev-parse", "HEAD"),
    tree: git("rev-parse", "HEAD^{tree}"),
    formatterTree: git("rev-parse", "HEAD:crates/vize_glyph/src"),
    cargoLockSha256: sha256(read("Cargo.lock")),
    observerSha256: sha256(read("crates/vize_glyph/examples/formatter_observe.rs")),
  };
  assert.equal(git("status", "--porcelain"), "", "campaign source must be committed and clean");
  const packs = [
    ["script", 6, 4, 0, 2],
    ["prepared", 82, 81, 1, 0],
    ["literal", 84, 84, 0, 0],
    ["literal-extra", 34, 34, 0, 0],
    ["vue-version", 14, 14, 0, 0],
    ["capture", 51, 31, 20, 0],
    ["capture-extra", 25, 24, 0, 1],
    ["capture-final", 4, 4, 0, 0],
  ] as const;
  const requiredLaws = Object.entries(audit.rustLaws).flatMap(([law, row]: [string, any]) =>
    row.currentWitnessRefs.map((ref: string) => {
      const witness = audit.witnessCatalog[ref];
      const owner = audit.sourceCatalog[witness.sourceRef];
      assert(owner.path.startsWith("crates/vize_glyph/"));
      const local = owner.path.slice("crates/vize_glyph/".length);
      const integration = /^tests\/([^/]+)\.rs$/.exec(local);
      assert(
        integration || local.startsWith("src/"),
        "law requires an explicit current test owner",
      );
      const prefix = integration
        ? ""
        : local
            .slice(4)
            .replace(/\.rs$/, "")
            .replace(/(?:^|\/)mod$/, "")
            .replace(/\/+/g, "::")
            .replace(/^lib$/, "");
      return {
        law,
        witness: ref,
        owner: owner.path,
        function: witness.function,
        target: integration?.[1] ?? "vize_glyph",
        kind: integration ? "test" : "lib",
        modulePrefix: prefix,
        currentOwnerSha256: sha256(read(owner.path)),
        currentFunctionSha256: sha256(
          retainedFormatterFunction(read(owner.path), witness.function),
        ),
        originalWitnesses: audit.rustLaws[law].originalWitnessRefs.map((originalRef: string) => {
          const original = audit.witnessCatalog[originalRef];
          const source = audit.sourceCatalog[original.sourceRef];
          const captures = source.revisions.map((revision: string) => {
            const result = spawnSync(
              "git",
              ["--no-replace-objects", "show", `${revision}:${source.path}`],
              { cwd: root, maxBuffer: 32 * 1024 * 1024 },
            );
            assert.equal(result.error, undefined);
            assert.equal(
              result.status,
              0,
              `original law unavailable: ${originalRef}:${revision}:${source.path}: ${result.stderr}`,
            );
            assert.equal(sha256(result.stdout), source.sha256);
            return {
              revision,
              gitBlob: git("rev-parse", `${revision}:${source.path}`),
              sourceSha256: source.sha256,
              bytes: result.stdout.length,
              functionSha256: sha256(retainedFormatterFunction(result.stdout, original.function)),
            };
          });
          return {
            witness: originalRef,
            function: original.function,
            owner: source.path,
            captures,
          };
        }),
      };
    }),
  );
  const cliBuild = [
    "build",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize",
    "--message-format=json-render-diagnostics",
  ];
  const selectedIntegrationTargets = [
    ...new Set<string>([
      ...requiredLaws.filter((law: any) => law.kind === "test").map((law: any) => law.target),
      "import_sorting",
      ...movedTargets,
    ]),
  ].sort();
  assert.equal(selectedIntegrationTargets.length, 11);
  const msrv = /^rust-version = "([^"]+)"$/m.exec(read("Cargo.toml").toString())?.[1];
  assert.equal(msrv, "1.95.0");
  const rustBuild = [
    "test",
    "--locked",
    "--profile",
    "ci",
    "-p",
    "vize_glyph",
    "--lib",
    ...selectedIntegrationTargets.flatMap((target) => ["--test", target]),
    "--no-run",
    "--message-format=json-render-diagnostics",
  ];
  const plan = {
    schema: "vize.formatter-history.campaign-plan",
    version: 1,
    source,
    originalAuditSha256: sha256(
      read("tests/_fixtures/differential/formatter-history/fix-history-audit.json"),
    ),
    corpusTree: git("rev-parse", "HEAD:tests/_fixtures/differential/formatter-history"),
    pinnedOriginal: { touchingCommits: 87, fixes: 56, requirements: 150 },
    registeredApi: 300,
    historicalRepeated: 286,
    freshExecution: "not executed by this plan",
    repeats: 2,
    packs: packs.map(([name, cases, bytes, errors, internal]) => ({
      name,
      cases,
      bytes,
      errors,
      internal,
      manifestSha256: sha256(
        read(`tests/_fixtures/differential/formatter-history/${name}-manifest.json`),
      ),
    })),
    requiredLaws,
    selectedIntegrationTargets,
    controlCapture: {
      canonicalPatchAuthority: canonicalPatchProfile,
      originalControls: originalControlPins(root, audit),
      currentSourcePins: currentControlPins(root, selectedIntegrationTargets),
      commands,
      msrv,
      benchmarkNames,
      actualSourceQualifier: "diagnostic child; reviewed parent 78cf receives no execution credit",
      performanceMetricCredit: false,
      applicableFormatterInstructionWorkload: null,
      remainingMetrics:
        "No Glyph workload/ceiling is registered in the current instruction harness. Smoke and correctness do not establish metrics; no budget is added or changed.",
      independentAcceptance: "pending complete hosted receipts and review",
    },
    engineeringControls: Object.entries(audit.controls).map(([id, c]: [string, any]) => ({
      id,
      contract: c.contract,
      state: "pending independent acceptance; no public output credit",
    })),
    cliCases: 5,
    cliBuild,
    rustBuild,
    importSortingRustTarget: { target: "import_sorting", sharedRegisteredCases: 0 },
    importSorting7258: "unfinished separate feature history; original captures provide no credit",
    nativeHandled: 0,
    wholeHistoryGate:
      "unfinished until execution, controls, new history, protected validation and actual merge",
    externalMutation: false,
    runnerFiles: [
      "formatter-history-campaign.ts",
      "campaign-plan.ts",
      "campaign-rust-laws.ts",
      "campaign-authority.ts",
      "formatter-import-sorting-capture.ts",
      "control-contract.ts",
      "campaign-controls.ts",
      "glyph-module-proof.ts",
      "original-control-patch-pins.json",
    ].map((name) => ({
      name,
      sha256: sha256(fs.readFileSync(path.join(import.meta.dirname, name))),
    })),
    diagnosticWorkflowSha256: sha256(
      fs.readFileSync(path.join(import.meta.dirname, "formatter-history-diagnostic.check.yml")),
    ),
  };
  assert.equal(
    plan.originalAuditSha256,
    "1f7577f236c035053a68857adad2e42657d31e5d3e9f3724a243d225ddfdfb33",
  );
  assert.equal(plan.corpusTree, "992f2dc8e18dd4e51ba0605edb325e94d4aa3401");
  const originalCorpusTree = "9cb36115546482b5cceccc687b3a3443d4f93069";
  const originalEntries = git("ls-tree", "-r", originalCorpusTree).split("\n");
  const currentEntries = git("ls-tree", "-r", plan.corpusTree).split("\n");
  const witnessPath = "source-witnesses/preserve_authored_content.cc87.txt";
  assert.deepEqual(
    currentEntries.filter((entry) => !entry.endsWith(`\t${witnessPath}`)),
    originalEntries,
    "all original corpus modes and blob identities must remain exact",
  );
  const sourceWitnessTransition = {
    originalCorpusTree,
    currentCorpusTree: plan.corpusTree,
    originalRevision: "cc87bb5960ea9e49e82672205df919de58bb4b24",
    owner: "crates/vize_glyph/tests/preserve_authored_content.rs",
    originalOwnerSha256: "9affca9435fc96cc470bf07840c71ed17844629c3361749484c893269855fa5d",
    currentOwnerSha256: "f6b9060822826c8a3942ce9a7c40f5876bf82fb7b2bfef81f10ff3c68aec8ec0",
    sourceAsset: `tests/_fixtures/differential/formatter-history/${witnessPath}`,
    sourceAssetBytes: 3907,
    role: "nonexecuting original source authority; no oracle, build or parent execution credit",
  };
  assert.equal(
    sha256(read(sourceWitnessTransition.owner)),
    sourceWitnessTransition.currentOwnerSha256,
  );
  assert.equal(
    sha256(read(sourceWitnessTransition.sourceAsset)),
    sourceWitnessTransition.originalOwnerSha256,
  );
  assert.equal(
    read(sourceWitnessTransition.sourceAsset).length,
    sourceWitnessTransition.sourceAssetBytes,
  );
  Object.assign(plan, { sourceWitnessTransition });
  return {
    plan,
    source,
    packs,
    requiredLaws,
    selectedIntegrationTargets,
    audit,
    cliBuild,
    rustBuild,
    git,
    sha256,
    fromRepo,
    loadFormatterHistoryAudit,
    validateFormatterHistoryExecution,
    loadFormatterApiManifest,
    validateFormatterApiReport,
    writeBuildReceipt,
    validateBuildReceipt,
    expectedBuildIdentity,
  };
}
