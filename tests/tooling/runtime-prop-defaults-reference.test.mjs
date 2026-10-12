import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { gunzipSync } from "node:zlib";
import { sha256, wholeJson } from "./lint-default-correctness-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/differential/lint/runtime-prop-defaults-8564");
const read = (name) => fs.readFileSync(path.join(fixture, name));
const json = (name) => JSON.parse(read(name));
const eq = assert.deepEqual,
  is = assert.equal;
const sum = (entries, field) => entries.reduce((n, entry) => n + entry[field].length, 0);
const frozen = {
  "source-law.json": "1412b36aa2ea22c31941c15058a61aa7b6e519965edebba161da15a7cc19fe62",
  "cases.json": "95453a7cecb70ab2deb0cd2a3585d218aa6caef7a9832078e98865dbc9930ad9",
  "default-route-controls.json": "3eac3263115863c9e1ff3e38b4140f10722c197d33117411aabcd145e006515e",
  "scoped-route-contract.json": "639ab77d684f5fbb20284136a3132057ef0497f899148523f0764241d9367cc0",
  "initial-route-preparation.json":
    "a415981bbb03ce4cf628f24b4501376275b26f2b963adb03d7fa5b93774692c4",
  "ownership-supplement.json": "5847eefb4138f8ebfab0335f3179c6f7242f3419a0ed6e08bed43519df080a7c",
  "read-only-original-provider-and-public-packets.json":
    "357840d6ec0c740e52f40e0ffd95d020477dc4abdad3acfb15c3e7820860d5a0",
  "frozen-originals-custody.json":
    "1d295171cedeafab0bc4eaf247440eeffd0a503630ce8a3871e4ac931775951d",
};
function bytesMatch(bytes, metadata, hash = "sha256", size = "bytes") {
  is(sha256(bytes), metadata[hash]);
  is(bytes.length, metadata[size]);
}
function apiPacket(entry, diagnostics) {
  return {
    filename: entry.filename,
    error_count: diagnostics.length,
    warning_count: 0,
    diagnostics,
  };
}

await test("runtime defaults retain full source-derived packets, ownership boundaries and original denominators", () => {
  const receipt = json("freeze-receipt.json");
  is(receipt.fixtureFiles.length, 53);
  for (const metadata of receipt.fixtureFiles) bytesMatch(read(metadata.path), metadata);
  bytesMatch(fs.readFileSync(path.join(root, receipt.referenceTest.path)), receipt.referenceTest);
  eq(receipt.scopes, {
    primarySources: 18,
    configuredPrimaryRoutes: 18,
    fullMinimalRoutes: 49,
    supplementalSources: 12,
  });
  for (const [name, digest] of Object.entries(frozen)) is(sha256(read(name)), digest, name);
  const law = json("source-law.json"),
    corpus = json("cases.json");
  const raw = json("read-only-original-provider-and-public-packets.json");
  const scoped = json("scoped-route-contract.json"),
    controls = json("default-route-controls.json");
  const initial = json("initial-route-preparation.json"),
    supplement = json("ownership-supplement.json");
  for (const metadata of law.frozenFiles) bytesMatch(read(metadata.path), metadata);
  for (const metadata of law.sources) bytesMatch(read(metadata.path), metadata);
  for (const metadata of scoped.sourceWitnesses) bytesMatch(read(metadata.path), metadata);
  eq(law.originalDenominators, [392, 357, 97, 443]);
  const custody = json("frozen-originals-custody.json");
  is(custody.authorities.length, 206);
  for (const metadata of custody.authorities) {
    bytesMatch(fs.readFileSync(path.join(root, metadata.path)), metadata);
  }
  eq(
    custody.packets.map(({ denominator }) => denominator),
    [392, 357, 443],
  );
  is(raw.helperSha256, sha256(read("original-observer.mjs.fixture")));
  is(raw.providerRuleSha256, sha256(read("pinned-provider-rule.js.fixture")));
  is(raw.originalManifestSha256, sha256(read("original-sources.json")));
  is(raw.authoredManifestSha256, sha256(read("authored-proposal.json")));
  is(raw.baseline.binarySha256, "9ad7cf956351265a152a9bbfa08b8c50c85dadacef66529474bbb6170267e254");
  is(raw.baseline.officialRun, 38081199501);
  eq(
    raw.packages.map(({ name, version }) => [name, version]),
    [
      ["eslint", "9.39.1"],
      ["eslint-plugin-vue", "10.9.2"],
      ["vue-eslint-parser", "10.4.1"],
      ["@typescript-eslint/parser", "8.65.0"],
      ["typescript", "6.0.3"],
    ],
  );
  const originals = json("original-sources.json"),
    authored = json("authored-proposal.json");
  is(originals.sources.length, 7);
  is(authored.cases.length, 11);
  is(corpus.cases.length, 18);
  eq(
    [
      sum(corpus.cases, "beforeDiagnostics"),
      sum(corpus.cases, "diagnostics"),
      sum(corpus.cases, "providerDiagnostics"),
      sum(corpus.cases, "providerForeignDiagnostics"),
    ],
    [0, 19, 23, 1],
  );
  eq(
    corpus.cases.filter((entry) => entry.conservativeOwnershipRefusal).map(({ id }) => id),
    law.ownership.primaryRefusals,
  );
  const directory = path.dirname(raw.runs[0].providerResults[0].filePath);
  assert.match(path.basename(directory), /^vize-runtime-props-readonly-/);
  is(raw.runs.length, 18);
  corpus.cases.forEach((entry, index) => {
    const source = read(entry.sourceFile);
    is(source.toString(), entry.source);
    is(sha256(source), entry.sourceSha256);
    if (index < 7) {
      const original = originals.sources[index];
      is(original.sha256, entry.sourceSha256);
      is(original.revision, "1acb8a3a7581e4db32ba0d529170c4669a2e1053");
      const blob = createHash("sha1")
        .update(`blob ${source.length}\0`)
        .update(source)
        .digest("hex");
      is(blob, original.apiBlob);
    } else {
      is(entry.source, authored.cases[index - 7].source);
    }
    for (const member of entry.members) {
      is(source.subarray(member.start, member.end).toString(), member.propertyText);
      is(
        source
          .subarray(member.key.start, member.key.end)
          .toString()
          .replaceAll("'", "")
          .replaceAll('"', ""),
        member.name,
      );
    }
    for (const finding of entry.diagnostics) {
      const member = entry.members.find(
        (member) => member.start === finding.start && member.end === finding.end,
      );
      assert.ok(member.requiredTrue && member.default && member.nativeEligible);
      is(finding.rule_name, corpus.rule);
      is(finding.severity, "error");
      is(finding.help, corpus.nativeContract.help);
      is(finding.fix, null);
      eq(finding.labels, [
        { message: "marked required here", ...member.required },
        { message: "default declared here", ...member.default },
      ]);
    }
    for (const finding of entry.providerDiagnostics) {
      const range = finding.suggestions[0].fix.range;
      is(entry.source.slice(...range), "true");
      is(finding.suggestions[0].fix.text, "false");
      is(finding.nodeType, "Property");
    }
    eq(entry.apiBeforePacket, apiPacket(entry, []));
    eq(entry.apiAfterPacket, apiPacket(entry, entry.diagnostics));
    const run = raw.runs[index],
      filename = path.join(directory, `${entry.id}-${entry.filename}`);
    is(run.id, entry.id);
    is(run.sourceSha256, entry.sourceSha256);
    is(run.source, entry.source);
    is(run.config, raw.config);
    eq(run.argv, [
      "lint",
      filename,
      "--format",
      "json",
      "--preset",
      "incremental",
      "--config",
      path.join(directory, "vize.config.json"),
    ]);
    is(run.status, 0);
    is(run.signal, null);
    is(run.error, null);
    is(run.stderr, "");
    const before = wholeJson({ ...entry, filename }, []);
    eq(run.actual, before);
    eq(JSON.parse(run.stdout), before);
    const messages = [...entry.providerDiagnostics, ...entry.providerForeignDiagnostics].sort(
      (a, b) => a.line - b.line || a.column - b.column,
    );
    eq(
      run.providerResults,
      [
        {
          filePath: filename,
          messages,
          suppressedMessages: [],
          errorCount: messages.length,
          fatalErrorCount: 0,
          warningCount: 0,
          fixableErrorCount: 0,
          fixableWarningCount: 0,
          ...(messages.length ? { source: entry.source } : {}),
          usedDeprecatedRules: [],
        },
      ],
      entry.id,
    );
  });
  const foreign = corpus.cases[1].providerForeignDiagnostics[0];
  is(foreign.ruleId, "@typescript-eslint/ban-ts-comment");
  is(foreign.message, "Definition for rule '@typescript-eslint/ban-ts-comment' was not found.");
  is(law.ownership.providerAuthorityForCompilerOwnership, false);

  is(scoped.targetCorpus.sha256, frozen[scoped.targetCorpus.path]);
  eq(scoped.apiBuilder.additionalRules, [corpus.rule]);
  is(scoped.apiBuilder.enabledRules, null);
  is(scoped.apiBuilder.preset, "incremental");
  is(scoped.apiBuilder.usesActualSfcDispatcher, true);
  is(scoped.cliConfig, raw.config);
  is(scoped.cases.length, 18);
  scoped.cases.forEach((route, index) => {
    const entry = corpus.cases[index];
    is(route.id, entry.id);
    is(route.sourceSha256, entry.sourceSha256);
    is(route.config, raw.config);
    eq(route.argv, [
      "lint",
      entry.filename,
      "--format",
      "json",
      "--preset",
      "incremental",
      "--config",
      "vize.config.json",
      "--locale",
      "en",
      "--help-level",
      "none",
    ]);
    eq(route.diagnostics, entry.diagnostics);
    eq(route.beforeApiPacket, apiPacket(entry, []));
    eq(route.afterApiPacket, apiPacket(entry, entry.diagnostics));
    eq(route.beforeCliPacket, wholeJson(entry, []));
    eq(route.afterCliPacket, wholeJson(entry, entry.diagnostics));
    is(route.beforeStatus, 0);
    is(route.afterStatus, entry.diagnostics.length ? 1 : 0);
  });
  is(initial.initialCases.sha256, frozen[initial.initialCases.path]);
  is(initial.initialSourceLaw.sha256, frozen[initial.initialSourceLaw.path]);
  is(initial.successor.sha256, frozen[initial.successor.path]);
  is(initial.executedInitialPreparationRoutes, 0);
  is(initial.recapturedExpectedVectors, 0);
  is(controls.targetCorpusSha256, frozen["cases.json"]);
  is(controls.cases.length, 7);
  for (const control of controls.cases) {
    const entry = corpus.cases.find((entry) => entry.id === control.id);
    is(control.sourceSha256, entry.sourceSha256);
    eq(control.companions, []);
    eq(
      control.routes.map(({ id }) => id),
      [
        "default",
        "recommended",
        "essential",
        "ecosystem",
        "incremental",
        "explicit-off",
        "n8n-projection",
      ],
    );
    control.routes.forEach((route, index) => {
      const findings = index < 4 ? entry.diagnostics : [];
      eq(route.apiPacket, apiPacket(entry, findings));
      eq(route.cliPacket, wholeJson(entry, findings));
      eq(route.beforeApiPacket, apiPacket(entry, []));
      eq(route.beforeCliPacket, wholeJson(entry, []));
      is(route.status, findings.length ? 1 : 0);
    });
  }
  is(supplement.primaryCorpus.sha256, frozen["cases.json"]);
  eq(supplement.summary, {
    cases: 12,
    beforeFindings: 0,
    afterFindings: 1,
    providerExecuted: false,
  });
  for (const entry of supplement.cases) {
    is(read(entry.sourceFile).toString(), entry.source);
    is(sha256(read(entry.sourceFile)), entry.sourceSha256);
    eq(entry.beforeDiagnostics, []);
    is(entry.diagnostics.length, entry.id === "runtime-vue-import" ? 1 : 0);
  }

  const compressed = read(law.matrix.path);
  bytesMatch(compressed, law.matrix);
  const bytes = gunzipSync(compressed);
  bytesMatch(bytes, law.matrix, "uncompressedSha256", "uncompressedBytes");
  const matrix = JSON.parse(bytes);
  is(matrix.project, law.matrix.project);
  is(matrix.revision, law.matrix.revision);
  eq(matrix.budget, law.matrix.budget);
  is(matrix.budget.verdict, "unusable");
  is(matrix.divergence.falsePositives.length, 43);
  is(matrix.divergence.falseNegatives.length, 60);
  is(
    matrix.divergence.falsePositives.filter((finding) => finding.ruleId === corpus.rule).length,
    42,
  );
  is(
    matrix.divergence.falseNegatives.filter((finding) => finding.ruleId === corpus.rule).length,
    58,
  );
});
