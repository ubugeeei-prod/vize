// Independent dev-only oracle for the owned latest-CLI configuration witnesses.
// Every call uses the complete explicit 51-rule projection, including controls.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { projection, root } from "./n8n-cli-config-inputs.mjs";

export const authoredCasesPath = path.join(
  root,
  "tests/_fixtures/differential/lint/n8n-cli-config/cases.json",
);
const pinnedProviders = {
  eslint: "10.4.1",
  "eslint-plugin-vue": "10.9.2",
  "vue-eslint-parser": "10.4.1",
  "@typescript-eslint/parser": "8.65.0",
};
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function loadAuthoredCases() {
  return JSON.parse(fs.readFileSync(authoredCasesPath, "utf8"));
}

export function errorPacket(error) {
  if (!(error instanceof Error)) return { thrown: errorValue(error) };
  // Error names, stacks, codes, causes and additional provider-owned fields all
  // remain in the receipt. No assertion can discard a provider failure.
  return {
    name: error.name,
    ...Object.fromEntries(
      Object.getOwnPropertyNames(error).map((name) => [name, errorValue(error[name])]),
    ),
  };
}

function errorValue(value) {
  if (value instanceof Error) return errorPacket(value);
  if (Array.isArray(value)) return value.map(errorValue);
  if (
    value !== null &&
    typeof value === "object" &&
    Object.getPrototypeOf(value) === Object.prototype
  ) {
    return Object.fromEntries(
      Object.getOwnPropertyNames(value).map((name) => [name, errorValue(value[name])]),
    );
  }
  return value;
}

/** Named recording step: serialize the whole provider packet; change filePath only. */
export function recordOraclePackets(
  rawPackets,
  recordedFilePath,
  actualFilePath = path.join(root, recordedFilePath),
) {
  const serializedRawPackets = JSON.parse(JSON.stringify(rawPackets));
  const packets = structuredClone(serializedRawPackets);
  for (const packet of packets) {
    // Unexpected provider paths remain untouched and fail the whole comparison.
    if (packet.filePath === actualFilePath) packet.filePath = recordedFilePath;
  }
  return { rawPackets, serializedRawPackets, packets };
}

function oracleRules(ruleOptions) {
  const rules = {};
  for (const [rule, severity] of Object.entries(projection.linter.rules)) {
    // These are explicit oracle identities in the committed owned projection.
    // Do not consult a preset/status list or synthesize an alias here.
    const oracleRule = projection.oracleRules[rule];
    const option = ruleOptions[rule];
    if (rule === "vue/component-name-in-template-casing") {
      const { casing, ...scope } = option;
      rules[oracleRule] = [severity, casing, ...(Object.keys(scope).length ? [scope] : [])];
    } else {
      rules[oracleRule] = option === undefined ? severity : [severity, option];
    }
  }
  return rules;
}

export async function captureAuthoredOracle({
  benchmarkManifest = path.join(root, "tools/benchmarks/scripts/package.json"),
  onRecorded = async () => {},
  fixture = loadAuthoredCases(),
  baseRuleOptions = projection.linter.ruleOptions,
  filePrefix = "tests/_fixtures/differential/lint/n8n-cli-config",
  officialVueBase = false,
} = {}) {
  const requireProvider = createRequire(benchmarkManifest);
  const captures = [];
  const providers = {};
  const loaded = {};
  let loadError;
  try {
    for (const name of Object.keys(pinnedProviders)) {
      const manifestPath = requireProvider.resolve(`${name}/package.json`);
      const entrypointPath = requireProvider.resolve(name);
      providers[name] = {
        version: requireProvider(`${name}/package.json`).version,
        publicEntrypoint: path
          .relative(path.dirname(manifestPath), entrypointPath)
          .replaceAll("\\", "/"),
        publicEntrypointSha256: sha256(fs.readFileSync(entrypointPath)),
        packageManifestSha256: sha256(fs.readFileSync(manifestPath)),
      };
      loaded[name] = requireProvider(name);
    }
  } catch (error) {
    loadError = errorPacket(error);
    await onRecorded({ phase: "provider-loading", rawError: error, error: loadError, providers });
  }

  const configurations = [
    { id: "latest-cli", ruleOptions: structuredClone(baseRuleOptions) },
    ...fixture.optionControls.map(({ id, ruleOptions }) => ({
      id,
      ruleOptions: { ...structuredClone(baseRuleOptions), ...ruleOptions },
    })),
  ].map(({ id, ruleOptions }) => ({ id, ruleOptions, rules: oracleRules(ruleOptions) }));
  const officialBase =
    officialVueBase && !loadError ? loaded["eslint-plugin-vue"].configs["flat/base"] : [];
  const baseReceipt = officialVueBase
    ? {
        officialVueBase: {
          export: "flat/base",
          rules: Object.fromEntries(
            officialBase.flatMap(({ rules = {} }) => Object.entries(rules)),
          ),
          processors: officialBase.flatMap(({ processor }) =>
            processor === undefined ? [] : [processor],
          ),
        },
      }
    : {};

  if (!loadError) {
    for (const configuration of configurations) {
      const selectedCases =
        configuration.id === "latest-cli"
          ? fixture.cases.map(({ id }) => id)
          : fixture.optionControls.find(({ id }) => id === configuration.id).caseIds;
      for (const caseId of selectedCases) {
        const sourceCase = fixture.cases.find(({ id }) => id === caseId);
        const recordedFilePath = `${filePrefix}/${caseId}.vue`;
        const capture = {
          configuration: configuration.id,
          caseId,
          sourceSha256: sha256(sourceCase.source),
        };
        try {
          const eslint = new loaded.eslint.ESLint({
            cwd: root,
            overrideConfigFile: true,
            overrideConfig: [
              ...officialBase,
              {
                files: ["**/*.vue"],
                languageOptions: {
                  parser: loaded["vue-eslint-parser"],
                  parserOptions: {
                    parser: loaded["@typescript-eslint/parser"],
                    ecmaVersion: "latest",
                    sourceType: "module",
                  },
                },
                plugins: { vue: loaded["eslint-plugin-vue"] },
                rules: configuration.rules,
              },
            ],
          });
          const rawPackets = await eslint.lintText(sourceCase.source, {
            filePath: path.join(root, recordedFilePath),
          });
          Object.assign(capture, recordOraclePackets(rawPackets, recordedFilePath));
        } catch (error) {
          capture.rawError = error;
          capture.error = errorPacket(error);
        }
        captures.push(capture);
        // Preserve the complete raw provider response/failure before any checks.
        await onRecorded(capture);
      }
    }
  }

  return {
    raw: { providers, configurations, loadError, captures, ...baseReceipt },
    recorded: {
      providers,
      ...baseReceipt,
      projectionSha256: sha256(
        fs.readFileSync(path.join(root, "tests/_fixtures/n8n-cli-adoption.json")),
      ),
      configurations,
      ...(loadError ? { loadError } : {}),
      captures: captures.map(
        ({
          rawPackets: _rawPackets,
          serializedRawPackets: _serializedRawPackets,
          rawError: _rawError,
          ...capture
        }) => capture,
      ),
    },
  };
}

export function assertAuthoredOracle(
  capture,
  expected = loadAuthoredCases().oracleCapture,
  fixture = loadAuthoredCases(),
) {
  assert.equal(capture.raw.loadError, undefined, "real oracle provider loading");
  for (const [name, version] of Object.entries(pinnedProviders)) {
    assert.equal(capture.recorded.providers[name]?.version, version, name);
  }
  for (const configuration of capture.recorded.configurations) {
    assert.equal(Object.keys(configuration.rules).length, 51, configuration.id);
  }
  for (const packet of capture.recorded.captures) {
    assert.equal(
      packet.error,
      undefined,
      `${packet.configuration}/${packet.caseId}: provider error`,
    );
    const expectedRuleIds =
      packet.configuration === "latest-cli"
        ? fixture.cases.find(({ id }) => id === packet.caseId).expectedOracleRuleIds
        : fixture.optionControls.find(({ id }) => id === packet.configuration)
            .expectedOracleRuleIds[packet.caseId];
    assert.deepEqual(
      packet.packets.flatMap(({ messages }) => messages.map(({ ruleId }) => ruleId)),
      expectedRuleIds,
      `${packet.configuration}/${packet.caseId}: complete findings`,
    );
  }
  assert.deepEqual(
    capture.recorded,
    expected,
    "complete independent ESLint packets and provenance",
  );
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const args = process.argv.slice(2);
  const valueAfter = (flag) => (args.includes(flag) ? args[args.indexOf(flag) + 1] : undefined);
  const receiptDir =
    valueAfter("--receipt-dir") ?? fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-cli-oracle-"));
  fs.mkdirSync(receiptDir, { recursive: true });
  const capture = await captureAuthoredOracle({
    benchmarkManifest: valueAfter("--benchmark-manifest"),
    onRecorded(packet) {
      const name = packet.phase ?? `${packet.configuration}--${packet.caseId}`;
      fs.writeFileSync(
        path.join(receiptDir, `${name}.json`),
        JSON.stringify(packet, null, 2) + "\n",
      );
    },
  });
  fs.writeFileSync(
    path.join(receiptDir, "complete-capture.json"),
    JSON.stringify(capture, null, 2) + "\n",
  );
  if (args.includes("--record")) {
    // Raw receipts already exist. Reject wrong pins, errors and foreign findings
    // before replacing a committed golden with a fresh provider recording.
    assertAuthoredOracle(capture, capture.recorded);
    const fixture = loadAuthoredCases();
    fixture.oracleCapture = capture.recorded;
    fs.writeFileSync(authoredCasesPath, JSON.stringify(fixture, null, 2) + "\n");
  }
  // Even --record cannot accept missing, foreign or parser-error findings.
  assertAuthoredOracle(capture);
  console.log(`Complete authored ESLint oracle passed; raw receipts: ${receiptDir}`);
}
