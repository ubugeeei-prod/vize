import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { after, test } from "node:test";
import {
  compileProbe,
  diagnosticReceipt,
  literalPath,
  literalPrefixes,
  probeVariant,
} from "../../tools/support/compat/github/canonical-corpus-io-probe.mjs";
import { gitObjectId } from "../../tools/support/compat/github/canonical-corpus-inventory.mjs";
import { parseNativeVector } from "../../tools/support/compat/github/canonical-corpus-native-walk.mjs";

const temporary = mkdtempSync(join(tmpdir(), "canonical-io-control-"));
after(() => rmSync(temporary, { recursive: true, force: true }));
const source = readFileSync("tests/davinci_test_support/src/corpus.rs");
const tools = compileProbe(source, join(temporary, "harness"));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const expected = (logical, bytes) => [
  logical,
  hash(bytes),
  bytes.length,
  gitObjectId("blob", bytes),
];
const rows = (directory) =>
  readFileSync(join(directory, "operations.jsonl"), "utf8")
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));

void test("serialized IO evidence retains source custody without becoming a snapshot receipt", () => {
  const identity = Object.freeze({
    schema: "vize.canonical-corpus-identity",
    version: 1,
    repository: "ubugeeei-prod/vize",
    sha: "a".repeat(40),
    tree: "b".repeat(40),
    runId: 123,
    attempt: 2,
  });
  const summaries = [
    { name: "full-short", success: true },
    { name: "full-manifest", success: false },
  ];
  const receipt = JSON.parse(JSON.stringify(diagnosticReceipt(identity, summaries)));
  assert.equal(receipt.schema, "vize.canonical-corpus-io-diagnostic");
  for (const key of ["repository", "sha", "tree", "runId", "attempt"])
    assert.equal(receipt[key], identity[key]);
  assert.deepEqual(receipt.summaries, summaries);
  assert.equal(receipt.success, false, "A failing original case cannot be reported as successful");
  assert.equal(identity.schema, "vize.canonical-corpus-identity");
});

void test("literal manifest spelling survives every source operation and prefix observation", () => {
  const workspace = join(temporary, "cycle");
  const root = `${workspace}/tests/_fixtures/_git`;
  mkdirSync(`${workspace}/tests/davinci_test_support`, { recursive: true });
  mkdirSync(`${root}/jellyfin-vue/packaging/deb`, { recursive: true });
  mkdirSync(`${root}/jellyfin-vue/src`, { recursive: true });
  const bytes = Buffer.from("<template>whole 日本語 𐀀\r\n</template>\r\n");
  writeFileSync(`${root}/jellyfin-vue/src/Whole.vue`, bytes);
  symlinkSync("../..", `${root}/jellyfin-vue/packaging/deb/root`);
  const vector = execFileSync(tools.collector, [root]);
  const paths = parseNativeVector(vector);
  assert(paths.length > 20, "Keep aliases through the observed native boundary");
  assert.equal(new Set(paths).size, paths.length);
  const longRoot = `${workspace}/tests/davinci_test_support/../../tests/_fixtures/_git`;
  assert.equal(literalPath(longRoot, paths[0]), `${longRoot}/${paths[0]}`);
  assert(
    literalPrefixes(literalPath(longRoot, paths[0])).some((prefix) =>
      prefix.endsWith("davinci_test_support/.."),
    ),
  );
  const summaries = [root, longRoot].map((spelling, index) =>
    probeVariant({
      binary: tools.binary,
      root: spelling,
      vector,
      expected: paths.map((path) => expected(path, bytes)),
      directory: join(workspace, `proof-${index}`),
    }),
  );
  for (const summary of summaries) {
    assert.equal(summary.selected, paths.length);
    assert.equal(summary.frameCount, paths.length * 4);
    assert.equal(summary.success, true);
    assert.equal(summary.nodeSequenceSha256, summary.rustSequenceSha256.file_open);
    assert.equal(summary.nodeSequenceSha256, summary.rustSequenceSha256.read_to_string);
  }
  assert.equal(summaries[0].nodeSequenceSha256, summaries[1].nodeSequenceSha256);
  const longRows = rows(join(workspace, "proof-1"));
  assert(
    longRows.every(
      (row) =>
        row.path.startsWith(`${longRoot}/`) &&
        row.node.matchesSnapshot &&
        Object.values(row.matches).every(Boolean),
    ),
  );
});

void test("missing selected sources stay in the whole vector and retain fatal original IO errors", () => {
  const root = join(temporary, "missing");
  mkdirSync(root);
  const bytes = Buffer.from("<template>retained</template>\n");
  writeFileSync(join(root, "Retained.vue"), bytes);
  const vector = Buffer.from("Missing.vue\0Retained.vue\0");
  const directory = join(temporary, "missing-proof");
  const summary = probeVariant({ binary: tools.binary, root, vector, directory });
  assert.equal(summary.selected, 2);
  assert.equal(summary.frameCount, 8);
  assert.equal(summary.success, false);
  assert.equal(summary.collectedVectorMatches, false);
  assert.equal(hash(readFileSync(join(directory, "rust-frames.bin"))), summary.framesSha256);
  const operations = rows(directory);
  assert.deepEqual(
    operations.map((row) => row.logical),
    ["Missing.vue", "Retained.vue"],
  );
  assert.equal(operations[0].node.code, "ENOENT");
  for (const operation of ["stat", "file_open", "read_to_string"]) {
    assert.equal(operations[0].rust[operation].status, "error");
    assert.match(operations[0].rust[operation].detail, /NotFound.*raw_os_error/);
  }
  assert.equal(operations[1].matches.file_open, true);
  assert.equal(operations[1].matches.read_to_string, true);
});

void test("invalid UTF8 remains a fatal read_to_string failure while whole binary bytes are retained", () => {
  const root = join(temporary, "utf8");
  mkdirSync(root);
  const bytes = Buffer.from([0x61, 0xff, 0x62, 0x0d, 0x0a]);
  writeFileSync(join(root, "Invalid.vue"), bytes);
  const directory = join(temporary, "utf8-proof");
  const summary = probeVariant({
    binary: tools.binary,
    root,
    vector: Buffer.from("Invalid.vue\0"),
    expected: [expected("Invalid.vue", bytes)],
    directory,
  });
  assert.equal(summary.success, false);
  assert.equal(summary.collectedVectorMatches, true);
  assert.equal(hash(readFileSync(join(directory, "rust-frames.bin"))), summary.framesSha256);
  const [operation] = rows(directory);
  assert.equal(operation.node.matchesSnapshot, true);
  assert.equal(operation.matches.file_open, true);
  assert.equal(operation.rust.read_to_string.status, "error");
  assert.match(operation.rust.read_to_string.detail, /InvalidData/);
});

void test("diagnostics cannot suppress the ordinary SSR worker or become an accepted gate failure", () => {
  const workflow = readFileSync(".github/workflows/davinci-canonical-corpus.yml", "utf8");
  const diagnostic = workflow.indexOf("- name: Diagnose complete original corpus source IO");
  const ssr = workflow.indexOf("- name: Run L4 SSR and pug L1 differential corpora");
  const fatal = workflow.indexOf("- name: Require complete source IO diagnostic success");
  const upload = workflow.indexOf("- name: Upload required observer evidence");
  assert(ssr > 0 && ssr < diagnostic && diagnostic < fatal && fatal < upload);
  assert.match(workflow.slice(diagnostic, fatal), /continue-on-error: true/);
  assert.match(
    workflow.slice(diagnostic, fatal),
    /always\(\).*steps\.corpus_snapshot\.outcome == 'success'/,
    "Collect diagnostic evidence after an original SSR failure without adding warmup before it",
  );
  assert.match(workflow.slice(fatal, upload), /always\(\).*steps\.corpus_io\.outcome != 'success'/);
  assert.match(workflow.slice(fatal, upload), /exit 1/);
  assert.match(
    workflow.slice(ssr, diagnostic),
    /cargo test -p vize_atelier_ssr[\s\S]*&& VIZE_DAVINCI_DIFFERENTIAL_CORPUS=tests\/_fixtures\/_git cargo test -p vize_l1_to_l2/,
  );
});
