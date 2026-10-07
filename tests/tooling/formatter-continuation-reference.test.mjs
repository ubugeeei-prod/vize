import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const base = path.join(root, "tests/_fixtures/differential/formatter-regressions");
const old = path.join(base, "directive-print-width-7876");
const current = path.join(base, "continuation-prefix-width-7876");
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");

test("original sixteen-case witness and the finite current corpus remain exact", () => {
  const originalBytes = fs.readFileSync(path.join(old, "corpus.json"));
  assert.equal(
    sha(originalBytes),
    "5a6bc66c6dc53678480e09cab6cd64d54c8f0cecaada29fe8b1b2fcd240dba2a",
  );
  const original = JSON.parse(originalBytes);
  assert.equal(original.cases.length, 16);
  for (const row of original.cases) {
    assert.equal(sha(fs.readFileSync(path.join(old, row.input))), row.inputSha256);
    assert.equal(sha(fs.readFileSync(path.join(old, row.output))), row.expectedSha256);
  }
  const currentBytes = fs.readFileSync(path.join(current, "corpus.json"));
  assert.equal(
    sha(currentBytes),
    "f120d2fb5696bdd0c6dc9efad9d3a91e5ddd54bb97db3ad38d58a7a3c4a08457",
  );
  const corpus = JSON.parse(currentBytes);
  assert.equal(corpus.cases.length, 11);
  const firstCorpusBytes = fs.readFileSync(
    path.join(current, "authored-ten-corpus.before-self-closing.json.txt"),
  );
  assert.equal(
    sha(firstCorpusBytes),
    "50dde628b6f3aa591ca23e5b9190364d0e78e8ee3556a6c9e416c01acedb90c4",
  );
  const firstCorpus = JSON.parse(firstCorpusBytes);
  assert.equal(firstCorpus.cases.length, 10);
  assert.deepEqual(corpus.cases.slice(0, 10), firstCorpus.cases);
  assert.deepEqual({ ...corpus, cases: firstCorpus.cases }, firstCorpus);
  assert.equal(corpus.cases[10].id, "continued-self-closing-img");
  for (const row of corpus.cases) {
    for (const [file, digest] of [
      [row.input, row.inputSha256],
      [row.output, row.outputSha256],
    ]) {
      const real = fs.realpathSync(path.join(current, file));
      assert(real.startsWith(fs.realpathSync(current) + path.sep));
      assert.equal(sha(fs.readFileSync(real)), digest);
    }
    assert.equal(row.nativeSupported, false);
  }
  assert.deepEqual(
    fs.readFileSync(path.join(current, "original-example.input")),
    fs.readFileSync(path.join(old, "original-example.input")),
  );
  assert.notEqual(
    sha(fs.readFileSync(path.join(current, "original-example.expected"))),
    sha(fs.readFileSync(path.join(old, "original-example.expected"))),
  );
});

test("whole stock contracts preserve the independent ten-case denominator", () => {
  const raw = fs.readFileSync(path.join(current, "contracts-index.json"));
  assert.equal(sha(raw), "cf6a10c97dcf83c411816432be301e3f30ba7f43cd406fbf2a6d891bcb33ddd1");
  const index = JSON.parse(raw);
  assert.equal(index.contracts.length, 10);
  const firstIndexBytes = fs.readFileSync(
    path.join(current, "authored-nine-contracts.before-self-closing.json.txt"),
  );
  assert.equal(
    sha(firstIndexBytes),
    "c6730442dd4d101cc99c26a2112e90d7b4b88238e7f1a9236aaea75d48874bdb",
  );
  const firstIndex = JSON.parse(firstIndexBytes);
  assert.equal(firstIndex.contracts.length, 9);
  assert.deepEqual(index.contracts.slice(0, 9), firstIndex.contracts);
  assert.deepEqual({ ...index, contracts: firstIndex.contracts }, firstIndex);
  assert.equal(index.contracts[9].id, "continued-self-closing-img");
  for (const row of index.contracts) {
    assert.equal(row.file, `${row.id}.stock-contract.json`);
    const bytes = fs.readFileSync(path.join(current, row.file));
    assert.equal(sha(bytes), row.sha256);
    const contract = JSON.parse(bytes);
    assert.equal(contract.case, row.id);
    assert.equal(contract.inputSha256, row.inputSha256);
    assert.equal(contract.expectedSourceSha256, row.expectedSourceSha256);
    assert.equal(contract.observations.length, 7);
  }
});
