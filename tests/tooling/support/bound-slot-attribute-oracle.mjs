import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { createRequire } from "node:module";

export const root = path.resolve(import.meta.dirname, "../../..");
export const corpusRoot = path.join(root, "tests/_fixtures/differential/lint/bound-slot-attribute");
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const pins = {
  eslint: "10.4.1",
  "eslint-plugin-vue": "10.9.2",
  "vue-eslint-parser": "10.4.1",
  "@typescript-eslint/parser": "8.65.0",
};

export function readCorpus() {
  return JSON.parse(fs.readFileSync(path.join(corpusRoot, "cases.json"), "utf8"));
}

function providerError(error) {
  if (!(error instanceof Error)) return { thrown: error };
  return {
    name: error.name,
    ...Object.fromEntries(
      Object.getOwnPropertyNames(error).map((key) => [
        key,
        error[key] instanceof Error ? providerError(error[key]) : error[key],
      ]),
    ),
  };
}

export async function captureOracle({
  benchmarkManifest = path.join(root, "tools/benchmarks/scripts/package.json"),
  onRecorded = async () => {},
} = {}) {
  const config = JSON.parse(fs.readFileSync(path.join(corpusRoot, "config.json"), "utf8"));
  const req = createRequire(benchmarkManifest);
  const providers = {};
  const loaded = {};
  let loadError;
  try {
    for (const [name, pin] of Object.entries(pins)) {
      const packageManifest = req.resolve(`${name}/package.json`);
      const entrypoint = req.resolve(name);
      const version = req(`${name}/package.json`).version;
      providers[name] = {
        version,
        publicEntrypoint: path.relative(path.dirname(packageManifest), entrypoint),
        publicEntrypointSha256: sha256(fs.readFileSync(entrypoint)),
        packageManifestSha256: sha256(fs.readFileSync(packageManifest)),
      };
      assert.equal(version, pin, name);
      loaded[name] = req(name);
    }
  } catch (error) {
    loadError = providerError(error);
    await onRecorded({ phase: "provider-loading", providers, providerError: loadError });
  }
  const selectedRules = {};
  for (const [nativeId, severity] of Object.entries(config.linter.rules)) {
    const identity = config.oracleRules[nativeId];
    assert.equal(typeof identity, "string", nativeId);
    const option = config.linter.ruleOptions[nativeId];
    selectedRules[identity] =
      option === undefined
        ? severity
        : [severity, nativeId === "vue/component-name-in-template-casing" ? option.casing : option];
  }
  assert.equal(Object.keys(selectedRules).length, 51);
  const captures = [];
  let mandatoryInfrastructure;
  if (!loadError) {
    const vue = loaded["eslint-plugin-vue"];
    const base = vue.configs["flat/base"].find((cfg) => cfg.name === "vue/base/setup-for-vue");
    assert.equal(base.processor, "vue/vue");
    assert.deepEqual(base.rules, {
      "vue/comment-directive": "error",
      "vue/jsx-uses-vars": "error",
    });
    mandatoryInfrastructure = { name: base.name, processor: base.processor, rules: base.rules };
    const eslint = new loaded.eslint.ESLint({
      cwd: root,
      overrideConfigFile: true,
      overrideConfig: [
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
          plugins: { vue },
          processor: base.processor,
          rules: { ...selectedRules, ...base.rules },
        },
      ],
    });
    for (const input of readCorpus().cases) {
      assert.equal(sha256(input.source), input.sourceSha256, input.id);
      const recordedPath = `tests/_fixtures/differential/lint/bound-slot-attribute/${input.filename}`;
      const actualPath = path.join(root, recordedPath);
      for (const repeat of [1, 2]) {
        const capture = { id: input.id, repeat, sourceSha256: input.sourceSha256 };
        try {
          capture.rawPackets = await eslint.lintText(input.source, { filePath: actualPath });
          const packets = JSON.parse(JSON.stringify(capture.rawPackets));
          // Named recording step: only the exact expected filePath is changed.
          for (const packet of packets) {
            if (packet.filePath === actualPath) packet.filePath = recordedPath;
          }
          capture.packets = packets;
        } catch (error) {
          capture.providerError = providerError(error);
        }
        await onRecorded(capture);
        captures.push(capture);
      }
    }
  }
  return {
    providers,
    selectedRules,
    ruleOptions: config.linter.ruleOptions,
    mandatoryInfrastructure,
    configSha256: sha256(fs.readFileSync(path.join(corpusRoot, "config.json"))),
    ...(loadError ? { loadError } : {}),
    captures: captures.map(({ rawPackets: _rawPackets, ...capture }) => capture),
  };
}

export function assertOracle(capture, expected) {
  assert.equal(capture.loadError, undefined, "provider load failure remains visible");
  assert.equal(capture.captures.length, readCorpus().cases.length * 2);
  for (const input of readCorpus().cases) {
    const repeated = capture.captures.filter((packet) => packet.id === input.id);
    assert.equal(repeated.length, 2, input.id);
    for (const packet of repeated) {
      assert.equal(packet.providerError, undefined, input.id);
      assert.equal(packet.packets.length, 1, input.id);
      assert.equal(packet.packets[0].fatalErrorCount, 0, input.id);
    }
    assert.deepEqual(repeated[0].packets, repeated[1].packets, `${input.id}: whole repeats`);
  }
  assert.deepEqual(capture, expected, "complete pinned independent packets and provenance");
}
