import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { formatterOraclePackage } from "../differential/formatter-cli-target.ts";
import {
  compareTemplateAstEquivalence,
  templateAstSemanticSignature,
} from "./support/sfc-equivalence.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const base = path.join(root, "tests/_fixtures/differential/formatter-regressions");
const old = path.join(base, "directive-print-width-7876");
const current = path.join(base, "continuation-prefix-width-7876");
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");

void test("original sixteen-case witness and the finite current corpus remain exact", () => {
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
    sha(
      Buffer.from(
        JSON.stringify(
          { ...JSON.parse(currentBytes), cases: JSON.parse(currentBytes).cases.slice(0, 11) },
          null,
          2,
        ) + "\n",
      ),
    ),
    "f120d2fb5696bdd0c6dc9efad9d3a91e5ddd54bb97db3ad38d58a7a3c4a08457",
  );
  const corpus = JSON.parse(currentBytes);
  assert.equal(corpus.cases.length, 14);
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

void test("whole stock contracts preserve the independent ten-case denominator", () => {
  const raw = fs.readFileSync(path.join(current, "contracts-index.json"));
  assert.equal(
    sha(
      Buffer.from(
        JSON.stringify(
          { ...JSON.parse(raw), contracts: JSON.parse(raw).contracts.slice(0, 10) },
          null,
          2,
        ) + "\n",
      ),
    ),
    "cf6a10c97dcf83c411816432be301e3f30ba7f43cd406fbf2a6d891bcb33ddd1",
  );
  const index = JSON.parse(raw);
  assert.equal(index.contracts.length, 13);
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

void test("directive framing preserves stock AST semantics and refuses whole semantic changes", () => {
  const ids = [
    "dynamic-bind-brackets-arrow-value-lf",
    "dynamic-event-brackets-modifier-lf",
    "dynamic-template-key-assignment-crlf",
  ];
  const corpus = JSON.parse(fs.readFileSync(path.join(current, "corpus.json")));
  const index = JSON.parse(fs.readFileSync(path.join(current, "contracts-index.json")));
  assert.deepEqual(
    corpus.cases.slice(11).map((row) => row.id),
    ids,
  );
  assert.deepEqual(
    index.contracts.slice(10).map((row) => row.id),
    ids,
  );
  const uiRequire = createRequire(
    formatterOraclePackage(
      new URL("./support/formatter-continuation-semantics.mjs", import.meta.url).href,
    ),
  );
  const vueRequire = createRequire(uiRequire.resolve("vue"));
  assert.equal(vueRequire("@vue/compiler-sfc/package.json").version, "3.5.35");
  const { parse } = uiRequire("vue/compiler-sfc");
  const ast = (source, id) => {
    const parsed = parse(source, { filename: id + ".vue", sourceMap: false });
    assert.deepEqual(parsed.errors, []);
    assert.ok(parsed.descriptor.template?.ast);
    return parsed.descriptor.template.ast;
  };
  const mutants = [
    [
      ["{title:0}", "{id:0}"],
      ['data-next="kept"', 'data-next="lost"'],
    ],
    [
      [".stop", ".prevent"],
      ["{click:0}", "{keyup:0}"],
    ],
    [
      ["framing retained", "framing lost"],
      ['="page"', '="page + 1"'],
    ],
  ];
  for (const [position, id] of ids.entries()) {
    const row = corpus.cases[11 + position];
    const original = fs.readFileSync(path.join(current, row.input), "utf8");
    const reference = fs.readFileSync(path.join(current, row.output), "utf8");
    const before = ast(original, id);
    assert.deepEqual(compareTemplateAstEquivalence(before, ast(reference, id)), []);
    assert.equal(
      templateAstSemanticSignature(before),
      templateAstSemanticSignature(ast(reference, id)),
    );
    for (const [from, to] of mutants[position]) {
      assert.ok(reference.includes(from));
      const changed = ast(reference.replace(from, to), id);
      assert.notDeepEqual(compareTemplateAstEquivalence(before, changed), []);
      assert.notEqual(templateAstSemanticSignature(before), templateAstSemanticSignature(changed));
    }
  }
});
