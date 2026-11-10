import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

import { CORSA_BIN, VIZE_BIN, ecosystemProductsApp } from "./apps.ts";
import {
  runVizeCheckJsonWithReceipt,
  type VizeCheckJson,
  type VizeCheckJsonReceipt,
} from "./vize-check.ts";

type ExpectedQualification = {
  readonly diagnostics: {
    readonly baseline: readonly string[];
    readonly negative: readonly string[];
    readonly positive: readonly string[];
  };
  readonly inputSha256: Readonly<Record<string, string>>;
  readonly virtualTsSha256: {
    readonly baseline: Readonly<Record<string, string>>;
    readonly negative: Readonly<Record<string, string>>;
    readonly positive: Readonly<Record<string, string>>;
  };
};

type ProviderIdentity = {
  readonly path: string;
  readonly realpath: string;
  readonly sha256: string;
  readonly version: {
    readonly status: number | null;
    readonly stderr: string;
    readonly stdout: string;
  };
};

const sourceFiles = [
  "ComposablesAndRouting.vue",
  "GraphAnd3D.vue",
  "MediaAndDataViz.vue",
  "NaiveUiOptionsApi.vue",
  "PrimeVueOptionsApi.vue",
  "UiLibraries.vue",
] as const;

export function qualifyEcosystemProducerDrift(repoRoot: string): void {
  const expectedPath = path.join(
    repoRoot,
    "tests",
    "_fixtures",
    "ecosystem-producer-qualification-11806.json",
  );
  const expected = JSON.parse(fs.readFileSync(expectedPath, "utf8")) as ExpectedQualification;
  const qualificationRoot = path.join(
    repoRoot,
    "target",
    "vize-tests",
    "ecosystem-producer-qualification",
  );
  const projectDir = path.join(qualificationRoot, "project");
  const receiptPath = path.join(
    repoRoot,
    "target",
    "differential",
    "ecosystem-producer-qualification",
    "receipt.json",
  );

  fs.rmSync(qualificationRoot, { force: true, recursive: true });
  fs.mkdirSync(qualificationRoot, { recursive: true });
  fs.cpSync(ecosystemProductsApp.cwd, projectDir, { recursive: true });
  fs.symlinkSync(
    path.join(repoRoot, "tests", "node_modules"),
    path.join(projectDir, "node_modules"),
  );

  const originals = Object.fromEntries(
    sourceFiles.map((file) => [file, readSource(projectDir, file)]),
  ) as Record<(typeof sourceFiles)[number], string>;
  const providersBefore = providerClosure();
  let baseline: VizeCheckJsonReceipt | undefined;
  let positive: VizeCheckJsonReceipt | undefined;
  let negative: VizeCheckJsonReceipt | undefined;
  let restored: VizeCheckJsonReceipt | undefined;

  try {
    baseline = run(projectDir);

    writeSource(
      projectDir,
      "GraphAnd3D.vue",
      replaceExactly(
        originals["GraphAnd3D.vue"],
        '<Panel position="top-left">',
        '<Panel v-if="selectedNode.data" position="top-left">',
      ),
    );
    writeSource(
      projectDir,
      "UiLibraries.vue",
      replaceExactly(
        originals["UiLibraries.vue"],
        '      <ASelect v-model:value="antValue" :options="libraryOptions" style="width: 240px" />',
        [
          "      <ASelect",
          '        v-model:value="antValue"',
          '        :options="libraryOptions"',
          '        :filter-option="(inputValue, option) => inputValue.length > 0 && option?.value !== undefined"',
          '        :filter-sort="(left, right) => String(left.value).localeCompare(String(right.value))"',
          '        style="width: 240px"',
          "      />",
        ].join("\n"),
      ),
    );
    positive = run(projectDir);

    writeSource(
      projectDir,
      "ComposablesAndRouting.vue",
      replaceExactly(
        originals["ComposablesAndRouting.vue"],
        '<span v-if="errors.email">{{ errors.email }}</span>',
        '<span v-if="errors.email.missing">{{ errors.email.toFixed() }}</span>',
      ),
    );
    writeSource(
      projectDir,
      "GraphAnd3D.vue",
      replaceExactly(
        readSource(projectDir, "GraphAnd3D.vue"),
        "selectedNodeId = $event.node.id",
        "selectedNodeId = $event.node.missing",
      ),
    );
    writeSource(
      projectDir,
      "MediaAndDataViz.vue",
      replaceExactly(
        replaceExactly(
          originals["MediaAndDataViz.vue"],
          ':item-size="44"',
          ":item-size=\"'forty-four'\"",
        ),
        "item.name",
        "item.missing",
      ),
    );
    writeSource(
      projectDir,
      "NaiveUiOptionsApi.vue",
      replaceExactly(
        originals["NaiveUiOptionsApi.vue"],
        "markReviewed(selectedLibrary)",
        "markReviewed($event.missing)",
      ),
    );
    writeSource(
      projectDir,
      "PrimeVueOptionsApi.vue",
      replaceExactly(
        originals["PrimeVueOptionsApi.vue"],
        "chooseAction(action.value)",
        "chooseAction($event.missing)",
      ),
    );
    writeSource(
      projectDir,
      "UiLibraries.vue",
      replaceExactly(
        replaceExactly(
          replaceExactly(
            readSource(projectDir, "UiLibraries.vue"),
            "inputValue.length > 0 && option?.value !== undefined",
            "inputValue.missing > 0 && option?.value !== undefined",
          ),
          "String(left.value).localeCompare(String(right.value))",
          "`${left.value}:${right.value}`",
        ),
        "promoteLibrary(antValue)",
        "promoteLibrary($event.missing)",
      ),
    );
    negative = run(projectDir);

    for (const file of sourceFiles) {
      writeSource(projectDir, file, originals[file]);
    }
    restored = run(projectDir);
  } finally {
    const providersAfter = providerClosure();
    const receipt = {
      inputs: sourceHashes(originals),
      providersAfter,
      providersBefore,
      runs: {
        baseline: baseline == null ? null : serializableRun(baseline),
        negative: negative == null ? null : serializableRun(negative),
        positive: positive == null ? null : serializableRun(positive),
        restored: restored == null ? null : serializableRun(restored),
      },
    };
    fs.mkdirSync(path.dirname(receiptPath), { recursive: true });
    fs.writeFileSync(receiptPath, JSON.stringify(receipt, null, 2) + "\n");
  }

  assert.ok(baseline != null && positive != null && negative != null && restored != null);
  assert.deepEqual(
    providerClosure(),
    providersBefore,
    "the physical provider closure changed mid-run",
  );
  assert.deepEqual(
    sourceHashes(originals),
    expected.inputSha256,
    "the six authored inputs drifted",
  );
  assertRun(baseline, 1, expected.diagnostics.baseline, expected.virtualTsSha256.baseline);
  assertRun(positive, 0, expected.diagnostics.positive, expected.virtualTsSha256.positive);
  assertRun(negative, 1, expected.diagnostics.negative, expected.virtualTsSha256.negative);
  assertRun(restored, 1, expected.diagnostics.baseline, expected.virtualTsSha256.baseline);
  assert.deepEqual(
    restored.parsed,
    baseline.parsed,
    "restoring every input must restore the packet",
  );
  assert.equal(
    restored.process.stdout,
    baseline.process.stdout,
    "restored stdout must be byte exact",
  );
  assert.equal(
    restored.process.stderr,
    baseline.process.stderr,
    "restored stderr must be byte exact",
  );
  assert.deepEqual(changedVirtualTsFiles(baseline.parsed, positive.parsed), [
    "src/GraphAnd3D.vue",
    "src/UiLibraries.vue",
  ]);
  assert.deepEqual(changedVirtualTsFiles(positive.parsed, negative.parsed), [
    "src/ComposablesAndRouting.vue",
    "src/GraphAnd3D.vue",
    "src/MediaAndDataViz.vue",
    "src/NaiveUiOptionsApi.vue",
    "src/PrimeVueOptionsApi.vue",
    "src/UiLibraries.vue",
  ]);
}

function run(cwd: string): VizeCheckJsonReceipt {
  return runVizeCheckJsonWithReceipt(cwd, ["src/**/*.vue"], {
    allowExitCodes: [0, 1],
    showVirtualTs: true,
    timeoutMs: 300_000,
  });
}

function assertRun(
  receipt: VizeCheckJsonReceipt,
  status: number,
  diagnostics: readonly string[],
  virtualTsSha256: Readonly<Record<string, string>>,
): void {
  assert.equal(receipt.process.status, status, receipt.process.stderr);
  assert.equal(receipt.process.signal, null);
  assert.match(receipt.process.stderr, /=== __vize_helpers\.d\.ts ===/);
  assert.equal(receipt.process.command, VIZE_BIN);
  assert.equal(JSON.stringify(receipt.parsed), receipt.process.stdout.trim());
  assert.deepEqual(flatDiagnostics(receipt.parsed), diagnostics);
  assert.deepEqual(generatedSourceHashes(receipt.parsed), virtualTsSha256);
}

function providerClosure(): { readonly cli: ProviderIdentity; readonly corsa: ProviderIdentity } {
  return {
    cli: providerIdentity(VIZE_BIN),
    corsa: providerIdentity(CORSA_BIN),
  };
}

function providerIdentity(providerPath: string): ProviderIdentity {
  const version = spawnSync(providerPath, ["--version"], { encoding: "utf8" });
  return {
    path: providerPath,
    realpath: fs.realpathSync(providerPath),
    sha256: sha256(fs.readFileSync(providerPath)),
    version: {
      status: version.status,
      stderr: version.stderr,
      stdout: version.stdout,
    },
  };
}

function serializableRun(receipt: VizeCheckJsonReceipt) {
  return {
    diagnostics: flatDiagnostics(receipt.parsed),
    process: receipt.process,
    virtualTsSha256: generatedSourceHashes(receipt.parsed),
  };
}

function flatDiagnostics(parsed: VizeCheckJson): string[] {
  return parsed.files.flatMap(({ diagnostics, file }) =>
    diagnostics.map((diagnostic) => `${file}\t${diagnostic}`),
  );
}

function generatedSourceHashes(parsed: VizeCheckJson): Record<string, string> {
  return Object.fromEntries(
    parsed.files.map(({ file, virtualTs }) => [file, sha256(normalizeVirtualTs(virtualTs ?? ""))]),
  );
}

function normalizeVirtualTs(virtualTs: string): string {
  return virtualTs.replace(
    /(?:[A-Za-z]:)?[^"\n]*[\\/]vize[\\/]canon[\\/]projects[\\/][0-9a-f]{64}/g,
    "<vize-canon-project>",
  );
}

function changedVirtualTsFiles(before: VizeCheckJson, after: VizeCheckJson): string[] {
  const beforeHashes = generatedSourceHashes(before);
  const afterHashes = generatedSourceHashes(after);
  return Object.keys(afterHashes).filter((file) => beforeHashes[file] !== afterHashes[file]);
}

function readSource(projectDir: string, file: (typeof sourceFiles)[number]): string {
  return fs.readFileSync(path.join(projectDir, "src", file), "utf8");
}

function writeSource(projectDir: string, file: (typeof sourceFiles)[number], source: string): void {
  fs.writeFileSync(path.join(projectDir, "src", file), source);
}

function replaceExactly(source: string, before: string, after: string): string {
  assert.equal(source.split(before).length - 1, 1, `expected one occurrence of ${before}`);
  return source.replace(before, after);
}

function sourceHashes(sources: Readonly<Record<string, string>>): Record<string, string> {
  return Object.fromEntries(
    Object.entries(sources).map(([file, source]) => [file, sha256(source)]),
  );
}

function sha256(value: string | Buffer): string {
  return createHash("sha256").update(value).digest("hex");
}
