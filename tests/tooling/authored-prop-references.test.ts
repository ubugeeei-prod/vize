import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import { authoredPropReferenceLocations } from "./support/authored-prop-references.ts";
import { exerciseAuthoredLspOracle } from "./support/real-project-lsp-authored-oracle.ts";
import {
  sortLocations,
  type Location,
  type OracleSession,
} from "./support/real-project-lsp-authored-utils.ts";
import type { LspAuthoredOracle } from "./support/real-project-lsp-report.ts";

type Source = { file: string; gitBlob: string; bytes: number; sha256: string; base64: string };
type Contract = {
  id: string;
  repository: string;
  revision: string;
  oracleJSON: string;
  sources: Source[];
};
const contracts = (
  JSON.parse(
    fs.readFileSync(
      new URL("support/authored-prop-reference-contracts.json", import.meta.url),
      "utf8",
    ),
  ) as { contracts: Contract[] }
).contracts;
const lew = "docs/layout/LewCodeHighlighter.vue";
const misskey = "packages/frontend/src/components/";
// Independently declared source positions, never copied from a session response or the helper.
const positions: Record<string, Array<[string, number, number]>> = {
  "lew-ui": [
    ["docs/components/LewCodeBox.vue", 66, 41],
    ["docs/guide/Install.vue", 11, 37],
    ["docs/guide/Locale.vue", 17, 37],
    ["docs/guide/QA.vue", 35, 37],
    ["docs/guide/QA.vue", 42, 37],
    ["docs/guide/Start.vue", 23, 37],
    ["docs/guide/Start.vue", 29, 37],
    ["docs/guide/Start.vue", 40, 37],
    [lew, 8, 2],
    [lew, 21, 13],
    ["docs/layout/LewDemoBox.vue", 215, 47],
    ["docs/layout/LewTypeCode.vue", 14, 39],
  ],
  misskey: [
    [`${misskey}MkCodeInline.stories.impl.ts`, 31, 2],
    [`${misskey}MkCodeInline.vue`, 6, 30],
    [`${misskey}MkCodeInline.vue`, 11, 1],
    [`${misskey}global/MkMfm.ts`, 397, 5],
  ],
};

void test("the two fixed source cohorts retain the complete current registry oracles and commit pins", () => {
  const registry = JSON.parse(
    fs.readFileSync(new URL("../_fixtures/vue-ecosystem-fixtures.json", import.meta.url), "utf8"),
  ) as {
    projects: Array<
      Pick<Contract, "id" | "repository" | "revision"> & { lspAuthoredOracle: LspAuthoredOracle }
    >;
  };
  for (const contract of contracts) {
    const projects = registry.projects.filter(({ id }) => id === contract.id);
    assert.equal(projects.length, 1);
    assert.equal(projects[0]!.repository, contract.repository);
    assert.equal(projects[0]!.revision, contract.revision);
    assert.deepEqual(projects[0]!.lspAuthoredOracle, JSON.parse(contract.oracleJSON));
  }
});

function expected(root: string, contract: Contract): Location[] {
  return positions[contract.id]!.map(([file, line, character]) => ({
    uri: pathToFileURL(path.join(root, file)).href,
    range: { start: { line, character }, end: { line, character: character + 4 } },
  }));
}

function fixture(contract: Contract) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-authored-prop-references-"));
  const oracle = JSON.parse(contract.oracleJSON) as LspAuthoredOracle;
  for (const source of contract.sources) {
    const body = Buffer.from(source.base64, "base64");
    assert.equal(body.length, source.bytes);
    assert.equal(createHash("sha256").update(body).digest("hex"), source.sha256);
    assert.equal(
      createHash("sha1").update(`blob ${body.length}\0`).update(body).digest("hex"),
      source.gitBlob,
    );
    const destination = path.join(root, source.file);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(destination, body);
  }
  // Misskey's unrelated boundary is never queried here. Stubs permit unchanged preflight;
  // the session stops at prepareRename and grants no later-stage/native acceptance.
  for (const [file, source] of [
    [oracle.componentBoundary.componentFile, oracle.componentBoundary.dependencyEdit.anchor],
    [oracle.componentBoundary.importerFile, oracle.componentBoundary.tagAnchor],
  ]) {
    const destination = path.join(root, file!);
    if (!fs.existsSync(destination)) {
      fs.mkdirSync(path.dirname(destination), { recursive: true });
      fs.writeFileSync(destination, source!);
    }
  }
  const full = expected(root, contract);
  const local = full.filter(
    ({ uri }) => uri === pathToFileURL(path.join(root, oracle.templateBinding.file)).href,
  );
  assert.equal(local.length, 2);
  const usage = local.find(
    ({ range }) => range.start.line === (contract.id === "lew-ui" ? 21 : 6),
  )!;
  const declaration = local.find((entry) => entry !== usage)!;
  return { root, oracle, full, local, usage, declaration };
}

type Fixture = ReturnType<typeof fixture>;
async function driverUntilRename(f: Fixture, response: Location[], accepts: boolean) {
  const laterStage = new Error("unexecuted prepareRename boundary");
  const requests: string[] = [];
  const open = new Set<string>();
  const session: OracleSession = {
    notify(method, params) {
      const uri = (params as { textDocument: { uri: string } }).textDocument.uri;
      if (method === "textDocument/didOpen") open.add(uri);
      else {
        assert.equal(method, "textDocument/didClose");
        open.delete(uri);
      }
    },
    async waitForNotification(method, predicate) {
      assert.equal(method, "textDocument/publishDiagnostics");
      const packet = { uri: f.usage.uri, version: 1, diagnostics: [] };
      assert.ok(predicate?.(packet));
      return packet;
    },
    async request(method, params) {
      requests.push(method);
      const query = params as {
        textDocument: { uri: string };
        position: unknown;
        context?: unknown;
      };
      assert.equal(query.textDocument.uri, f.usage.uri);
      assert.deepEqual(query.position, f.usage.range.start);
      if (method === "textDocument/hover")
        return {
          contents: f.oracle.templateBinding.hoverContains.join("\n"),
          range: f.usage.range,
        };
      if (method === "textDocument/definition") return [f.declaration];
      if (method === "textDocument/references") {
        assert.deepEqual(query.context, { includeDeclaration: true });
        return response;
      }
      assert.equal(method, "textDocument/prepareRename");
      throw laterStage;
    },
  };
  await assert.rejects(
    () => exerciseAuthoredLspOracle(session, f.root, f.oracle),
    (error) =>
      accepts
        ? error === laterStage
        : error instanceof assert.AssertionError &&
          error.message.includes("references must cover exactly"),
  );
  assert.deepEqual(requests, [
    "textDocument/hover",
    "textDocument/definition",
    "textDocument/references",
    ...(accepts ? ["textDocument/prepareRename"] : []),
  ]);
  assert.equal(open.size, 0);
}

for (const contract of contracts) {
  void test(`${contract.id}: fixed complete source/import/UTF-16 prop group reaches the unchanged later boundary`, async () => {
    const f = fixture(contract);
    try {
      for (const entry of f.full) {
        const line = fs.readFileSync(fileURLToPath(entry.uri), "utf8").split("\n")[
          entry.range.start.line
        ]!;
        assert.equal(
          line.slice(entry.range.start.character, entry.range.end.character),
          f.oracle.templateBinding.symbol,
        );
      }
      assert.deepEqual(
        sortLocations(authoredPropReferenceLocations(f.root, f.oracle, f.local)),
        sortLocations(f.full),
      );
      await driverUntilRename(f, [...f.full].reverse(), true);
      await driverUntilRename(f, f.local, false);
      if (contract.id === "misskey") {
        const mfm = f.full[3]!;
        const lines = fs.readFileSync(fileURLToPath(mfm.uri), "utf8").split("\n");
        assert.equal(lines[389], lines[397], "genuine same-spelling MkCode consumer");
        const collision = {
          uri: mfm.uri,
          range: { start: { line: 389, character: 5 }, end: { line: 389, character: 9 } },
        };
        await driverUntilRename(f, [...f.full, collision], false);
        const wrongValue = structuredClone(f.full);
        const character = lines[397]!.lastIndexOf("code");
        wrongValue[3]!.range = {
          start: { line: 397, character },
          end: { line: 397, character: character + 4 },
        };
        await driverUntilRename(f, wrongValue, false);
      }
      await driverUntilRename(f, f.full, true);
    } finally {
      fs.rmSync(f.root, { recursive: true, force: true });
    }
  });

  void test(`${contract.id}: whole driver refuses every missing, duplicate, URI and four-coordinate mutation`, async () => {
    const f = fixture(contract);
    try {
      for (const [index, target] of f.full.entries()) {
        await driverUntilRename(
          f,
          f.full.filter((_, at) => at !== index),
          false,
        );
        await driverUntilRename(f, [...f.full, target], false);
        for (const uri of ["file:///foreign/Other.vue", `${target.uri}.ts`]) {
          const changed = structuredClone(f.full);
          changed[index]!.uri = uri;
          await driverUntilRename(f, changed, false);
        }
        for (const edge of ["start", "end"] as const) {
          for (const coordinate of ["line", "character"] as const) {
            const changed = structuredClone(f.full);
            changed[index]!.range[edge][coordinate] += 1;
            await driverUntilRename(f, changed, false);
          }
        }
        await driverUntilRename(f, f.full, true);
      }
      await driverUntilRename(
        f,
        [...f.full, { ...f.full[0]!, uri: pathToFileURL(path.join(f.root, "Other.vue")).href }],
        false,
      );
      await driverUntilRename(f, f.full, true);
    } finally {
      fs.rmSync(f.root, { recursive: true, force: true });
    }
  });

  void test(`${contract.id}: each whole source and config is mandatory and immutable`, () => {
    const f = fixture(contract);
    try {
      for (const source of contract.sources) {
        const file = path.join(f.root, source.file);
        const original = fs.readFileSync(file);
        fs.unlinkSync(file);
        assert.throws(() => authoredPropReferenceLocations(f.root, f.oracle, f.local));
        fs.writeFileSync(file, Buffer.concat([original, Buffer.from("\n")]));
        assert.throws(() => authoredPropReferenceLocations(f.root, f.oracle, f.local));
        fs.unlinkSync(file);
        const other = `${file}.body`;
        fs.writeFileSync(other, original);
        fs.symlinkSync(other, file);
        assert.throws(
          () => authoredPropReferenceLocations(f.root, f.oracle, f.local),
          /regular file/,
        );
        fs.unlinkSync(file);
        fs.renameSync(other, file);
        fs.chmodSync(file, 0o755);
        assert.throws(
          () => authoredPropReferenceLocations(f.root, f.oracle, f.local),
          /non-executable/,
        );
        fs.chmodSync(file, 0o644);
        assert.deepEqual(
          sortLocations(authoredPropReferenceLocations(f.root, f.oracle, f.local)),
          sortLocations(f.full),
        );
      }
      const directory = path.dirname(path.join(f.root, contract.sources[0]!.file));
      const moved = `${directory}.source`;
      fs.renameSync(directory, moved);
      fs.symlinkSync(moved, directory, "dir");
      assert.throws(
        () => authoredPropReferenceLocations(f.root, f.oracle, f.local),
        /physical source path/,
      );
      fs.unlinkSync(directory);
      fs.renameSync(moved, directory);
      const badLocal = structuredClone(f.local);
      badLocal[0]!.range.end.character += 1;
      assert.throws(
        () => authoredPropReferenceLocations(f.root, f.oracle, badLocal),
        /whole original local pair/,
      );
      const changedOracle = structuredClone(f.oracle);
      changedOracle.templateBinding.renameTo += "Changed";
      assert.throws(
        () => authoredPropReferenceLocations(f.root, changedOracle, f.local),
        /entire original authored oracle/,
      );
      const changedBoundary = structuredClone(f.oracle);
      changedBoundary.componentBoundary.completionItemCount += 1;
      assert.throws(
        () => authoredPropReferenceLocations(f.root, changedBoundary, f.local),
        /entire original authored oracle/,
      );
      assert.deepEqual(
        sortLocations(authoredPropReferenceLocations(f.root, f.oracle, f.local)),
        sortLocations(f.full),
      );
    } finally {
      fs.rmSync(f.root, { recursive: true, force: true });
    }
  });
}

void test("mixed cohorts refuse; every other binding retains the original pair without fixture reads", () => {
  const first = fixture(contracts[0]!);
  const second = fixture(contracts[1]!);
  try {
    assert.throws(() => authoredPropReferenceLocations(first.root, second.oracle, second.local));
    assert.throws(() => authoredPropReferenceLocations(second.root, first.oracle, first.local));
    for (const f of [first, second]) {
      const changed = structuredClone(f.oracle);
      changed.templateBinding.symbol = "otherBinding";
      assert.equal(authoredPropReferenceLocations("absent fixture", changed, f.local), f.local);
      changed.templateBinding.file = "Other.vue";
      assert.equal(authoredPropReferenceLocations("absent fixture", changed, f.local), f.local);
      assert.deepEqual(
        sortLocations(authoredPropReferenceLocations(f.root, f.oracle, f.local)),
        sortLocations(f.full),
      );
    }
  } finally {
    fs.rmSync(first.root, { recursive: true, force: true });
    fs.rmSync(second.root, { recursive: true, force: true });
  }
});
