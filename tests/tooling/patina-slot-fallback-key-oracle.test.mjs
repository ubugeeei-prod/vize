import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

const repository = path.resolve(import.meta.dirname, "../..");
const fixturePath = path.join(
  repository,
  "crates/vize_patina/tests/fixtures/slot-fallback-key/cases.json",
);
const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const errorPacket = (error) =>
  Object.fromEntries(Object.getOwnPropertyNames(error).map((key) => [key, error[key]]));

function providerInformation(requireProvider, name) {
  const entry = requireProvider.resolve(name);
  let directory = path.dirname(entry);
  while (directory !== path.dirname(directory)) {
    const manifest = path.join(directory, "package.json");
    if (fs.existsSync(manifest)) {
      const bytes = fs.readFileSync(manifest);
      const metadata = JSON.parse(bytes);
      if (metadata.name === name) {
        return {
          version: metadata.version,
          publicEntrypoint: path.relative(directory, entry),
          publicEntrypointSha256: sha256(fs.readFileSync(entry)),
          packageManifestSha256: sha256(bytes),
        };
      }
    }
    directory = path.dirname(directory);
  }
  throw new Error(`Cannot locate the public package manifest for ${name}`);
}

// Named recording step: normalize only the known ephemeral absolute filePath.
// Every other packet field, unknown path, fix, suppression and count is retained.
function recordPackets(packets, absolutePath, stablePath) {
  return packets.map((packet) =>
    packet.filePath === absolutePath ? { ...packet, filePath: stablePath } : packet,
  );
}

async function capture() {
  const receipts = fs.mkdtempSync(path.join(os.tmpdir(), "vize-slot-key-oracle-"));
  const raw = { providers: {}, configuration: null, loadError: null, captures: [] };
  const recorded = { providers: {}, configuration: null, captures: [] };
  let eslint;
  try {
    const requireProvider = createRequire(
      process.env.VIZE_SLOT_KEY_BENCHMARK_MANIFEST ??
        path.join(repository, "tools/benchmarks/scripts/package.json"),
    );
    for (const name of Object.keys(fixture.providers)) {
      raw.providers[name] = providerInformation(requireProvider, name);
    }
    recorded.providers = raw.providers;
    const vue = requireProvider("eslint-plugin-vue");
    const base = vue.configs["flat/base"].find(
      (config) => config.name === "vue/base/setup-for-vue",
    );
    raw.configuration = {
      selectedRules: fixture.selectedRules,
      mandatoryInfrastructure: { name: base.name, processor: base.processor, rules: base.rules },
      languageOptions: {
        parser: "vue-eslint-parser",
        parserOptions: {
          parser: "@typescript-eslint/parser",
          ecmaVersion: "latest",
          sourceType: "module",
        },
      },
    };
    recorded.configuration = raw.configuration;
    eslint = new (requireProvider("eslint").ESLint)({
      cwd: repository,
      overrideConfigFile: true,
      overrideConfig: [
        {
          files: ["**/*.vue"],
          languageOptions: {
            parser: requireProvider("vue-eslint-parser"),
            parserOptions: {
              parser: requireProvider("@typescript-eslint/parser"),
              ecmaVersion: "latest",
              sourceType: "module",
            },
          },
          plugins: { vue },
          processor: base.processor,
          rules: { ...fixture.selectedRules, ...base.rules },
        },
      ],
    });
  } catch (error) {
    raw.loadError = errorPacket(error);
  }
  if (eslint) {
    for (const example of fixture.cases) {
      const stablePath = `owned/${example.filename}`;
      const absolutePath = path.join(repository, stablePath);
      const result = {
        caseId: example.id,
        source: example.source,
        rawPackets: null,
        providerError: null,
      };
      try {
        result.rawPackets = await eslint.lintText(example.source, { filePath: absolutePath });
      } catch (error) {
        result.providerError = errorPacket(error);
      }
      fs.appendFileSync(path.join(receipts, "packets.jsonl"), JSON.stringify(result) + "\n");
      raw.captures.push(result);
      recorded.captures.push({
        caseId: example.id,
        packets: result.rawPackets
          ? recordPackets(result.rawPackets, absolutePath, stablePath)
          : null,
        providerError: result.providerError,
      });
    }
  }
  // Preserve complete provider errors and packets before any qualification assertion.
  fs.writeFileSync(path.join(receipts, "raw.json"), JSON.stringify(raw, null, 2) + "\n");
  return { raw, recorded, receipts };
}

function validateCapture({ raw }) {
  assert.equal(raw.loadError, null);
  assert.deepEqual(raw.providers, fixture.providers);
  assert.equal(Object.keys(raw.configuration.selectedRules).length, 51);
  assert.equal(raw.configuration.mandatoryInfrastructure.processor, "vue/vue");
  assert.deepEqual(raw.configuration.mandatoryInfrastructure.rules, {
    "vue/comment-directive": "error",
    "vue/jsx-uses-vars": "error",
  });
  assert.equal(raw.captures.length, fixture.cases.length);
  for (const result of raw.captures) assert.equal(result.providerError, null, result.caseId);
}

if (process.argv.includes("--record")) {
  const result = await capture();
  process.stdout.write(`Complete raw provider receipt: ${result.receipts}/raw.json\n`);
  validateCapture(result);
  fixture.oracleCapture = result.recorded;
  fs.writeFileSync(fixturePath, JSON.stringify(fixture, null, 2) + "\n");
} else {
  test("slot fallback key cases preserve complete pinned official Vue base packets", async (t) => {
    const result = await capture();
    t.diagnostic(`Complete raw provider receipt: ${result.receipts}/raw.json`);
    validateCapture(result);
    assert.deepEqual(result.recorded, fixture.oracleCapture);
  });
}
