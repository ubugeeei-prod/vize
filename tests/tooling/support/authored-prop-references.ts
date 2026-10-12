import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import { offsetToPosition } from "./lsp/assertions.ts";
import type { LspAuthoredOracle } from "./real-project-lsp-report.ts";
import {
  anchoredSymbolRange,
  readOracleDocument,
  sortLocations,
  uniqueAnchorOffset,
  type Location,
} from "./real-project-lsp-authored-utils.ts";

type Source = { file: string; gitBlob: string; bytes: number; sha256: string; base64: string };
type Consumer = {
  file: string;
  line: string;
  marker: string;
  context: string;
  kind: "template-prop" | "story-args" | "render-prop";
};
type Contract = {
  id: string;
  revision: string;
  component: string;
  config: string;
  oracleJSON: string;
  oracleSHA256: string;
  sources: Source[];
  consumers: Consumer[];
};
type ProjectConfig = {
  compilerOptions: { baseUrl?: string; paths: Record<string, string[]> };
  include: string[];
  exclude?: string[];
};
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");
const table = fs.readFileSync(new URL("authored-prop-reference-contracts.json", import.meta.url));
assert.equal(
  sha(table),
  "c2f1593ac9752cd214feaba3c044ee4bf4b0818be2a0b579a2e2d12a55f5498c",
  "complete fixed two-case source authority",
);
const contracts = (JSON.parse(table.toString()) as { contracts: Contract[] }).contracts;
assert.equal(contracts.length, 2);
assert.deepEqual(
  contracts.map(({ id }) => id),
  ["lew-ui", "misskey"],
);

function sourceDocuments(workspaceDir: string, contract: Contract) {
  const documents = new Map<string, ReturnType<typeof readOracleDocument>>();
  const physicalRoot = fs.realpathSync(workspaceDir);
  for (const source of contract.sources) {
    assert.equal(documents.has(source.file), false, "source authority must be unique");
    const document = readOracleDocument(workspaceDir, source.file);
    const absolute = path.resolve(workspaceDir, source.file);
    const stat = fs.lstatSync(absolute);
    assert.ok(stat.isFile(), "source authority must be a regular file");
    assert.equal(stat.mode & 0o111, 0, "source authority must be non-executable");
    assert.equal(
      fs.realpathSync(absolute),
      path.resolve(physicalRoot, source.file),
      "declared physical source path",
    );
    const bytes = fs.readFileSync(absolute);
    assert.ok(Buffer.from(document.source).equals(bytes), "complete UTF-8 source bytes");
    assert.equal(bytes.length, source.bytes, source.file);
    assert.equal(sha(bytes), source.sha256, source.file);
    assert.equal(
      createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
      source.gitBlob,
      source.file,
    );
    assert.ok(bytes.equals(Buffer.from(source.base64, "base64")), source.file);
    documents.set(source.file, document);
  }
  return documents;
}

function importedComponent(
  workspaceDir: string,
  contract: Contract,
  file: string,
  source: string,
  bindingFile: string,
  config: ProjectConfig,
) {
  const imports = [...source.matchAll(/^import (\w+) from (['"])([^'"\n]+)\2;?$/gmu)].filter(
    (match) => match[1] === contract.component,
  );
  assert.equal(imports.length, 1, "exact direct component import");
  const specifier = imports[0]![3]!;
  let resolved: string;
  if (specifier.startsWith(".")) {
    resolved = path.resolve(workspaceDir, path.dirname(file), specifier);
  } else {
    assert.equal(contract.id, "misskey");
    assert.ok(specifier.startsWith("@/"), "only the fixed frontend alias");
    assert.deepEqual(config.compilerOptions.paths["@/*"], ["./src/*"]);
    resolved = path.resolve(workspaceDir, path.dirname(contract.config), "src", specifier.slice(2));
  }
  assert.equal(resolved, path.resolve(workspaceDir, bindingFile), "same authored component");
}

/** Construct only the two source-declared prop groups, never from a native response. */
export function authoredPropReferenceLocations(
  workspaceDir: string,
  oracle: LspAuthoredOracle,
  originalLocal: Location[],
): Location[] {
  const contract = contracts.find(({ oracleJSON }) => {
    const fixed = (JSON.parse(oracleJSON) as LspAuthoredOracle).templateBinding;
    return (
      fixed.file === oracle.templateBinding.file && fixed.symbol === oracle.templateBinding.symbol
    );
  });
  if (!contract) return originalLocal;
  assert.equal(sha(contract.oracleJSON), contract.oracleSHA256);
  assert.deepEqual(oracle, JSON.parse(contract.oracleJSON), "entire original authored oracle");
  const binding = oracle.templateBinding;
  const documents = sourceDocuments(workspaceDir, contract);
  const primary = documents.get(binding.file)!;
  const local = [
    {
      range: anchoredSymbolRange(primary.source, binding.usageAnchor, binding.symbol, "usage"),
      uri: primary.uri,
    },
    {
      range: anchoredSymbolRange(
        primary.source,
        binding.declarationAnchor,
        binding.symbol,
        "declaration",
      ),
      uri: primary.uri,
    },
  ];
  assert.deepEqual(sortLocations(originalLocal), sortLocations(local), "whole original local pair");
  // Only the complete fixed Misskey config needs its known trailing comma removed.
  const config = JSON.parse(
    documents.get(contract.config)!.source.replace(/,(\s*[}\]])/gu, "$1"),
  ) as ProjectConfig;
  if (contract.id === "lew-ui") {
    assert.equal(config.compilerOptions.baseUrl, "./");
    assert.deepEqual(config.compilerOptions.paths["docs/*"], ["./docs/*"]);
    assert.deepEqual(config.include, ["lib", "docs", "public"]);
    assert.ok(
      contract.sources.every(({ file }) => file === contract.config || file.startsWith("docs/")),
    );
  } else {
    assert.ok(
      config.include.includes("./src/**/*.ts") && config.include.includes("./src/**/*.vue"),
    );
    assert.deepEqual(config.exclude, ["./src/**/*.stories.ts"]);
    assert.ok(
      contract.sources.every(
        ({ file }) => file === contract.config || file.startsWith("packages/frontend/src/"),
      ),
    );
  }
  const expected = [...originalLocal];
  for (const consumer of contract.consumers) {
    const document = documents.get(consumer.file)!;
    importedComponent(workspaceDir, contract, consumer.file, document.source, binding.file, config);
    const contextStart = uniqueAnchorOffset(
      document.source,
      consumer.context,
      "whole component consumer context",
    );
    if (consumer.kind === "template-prop") {
      assert.ok(consumer.line.includes(`<${contract.component} `));
      assert.equal(consumer.marker, `${binding.symbol}=`);
    } else if (consumer.kind === "story-args") {
      uniqueAnchorOffset(
        document.source,
        `template: '<${contract.component} v-bind="props"/>'`,
        "story template",
      );
      uniqueAnchorOffset(
        document.source,
        `} satisfies StoryObj<typeof ${contract.component}>;`,
        "typed story",
      );
      assert.ok(consumer.context.startsWith("\targs: {\n"));
    } else {
      assert.equal(consumer.kind, "render-prop");
      assert.ok(consumer.context.startsWith(`return [h(${contract.component}, {\n`));
    }
    const start =
      contextStart +
      uniqueAnchorOffset(consumer.context, consumer.line, "whole prop consumer line") +
      uniqueAnchorOffset(consumer.line, consumer.marker, "prop key");
    assert.equal(document.source.slice(start, start + binding.symbol.length), binding.symbol);
    expected.push({
      uri: document.uri,
      range: {
        start: offsetToPosition(document.source, start),
        end: offsetToPosition(document.source, start + binding.symbol.length),
      },
    });
  }
  assert.equal(expected.length, contract.id === "lew-ui" ? 12 : 4);
  assert.equal(new Set(expected.map((entry) => JSON.stringify(entry))).size, expected.length);
  return expected;
}
