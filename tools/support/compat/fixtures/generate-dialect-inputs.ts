import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const corpusDir = "tests/_fixtures/differential/dialect-inputs";
const grammarPath = path.join(root, corpusDir, "grammar.json");
const inventoryPath = path.join(root, corpusDir, "inventory.json");
const generatedDir = path.join(root, corpusDir, "generated");

interface Witness {
  readonly file: string;
  readonly selector: string;
}

interface GrammarRow {
  readonly id: string;
  readonly production: string;
  readonly selector: { readonly vueVersion: string; readonly templateSyntax: string };
  readonly dialects: readonly string[];
  readonly expectedParserMode: string;
  readonly witness: Witness;
  readonly tokens: Readonly<Record<string, string>>;
}

interface Grammar {
  readonly schema: string;
  readonly version: number;
  readonly cases: readonly GrammarRow[];
}

const productions: Readonly<Record<string, (tokens: Readonly<Record<string, string>>) => string>> =
  {
    list: ({ loop = "", event = "" }) => `<ul>\n  <li ${loop} ${event}>{{ item }}</li>\n</ul>\n`,
    "raw-html": () => "<div>{{{ html }}}</div>\n",
    "attribute-interpolation": () => '<a href="{{ url }}">Open</a>\n',
    "adjacent-attributes": () => '<div id="a"class="b"></div>\n',
    "void-close": () => "<img></img>\n",
  };

export function buildDialectInputs() {
  // The checked-in grammar; every row is validated below before it is rendered.
  const grammar = JSON.parse(fs.readFileSync(grammarPath, "utf8")) as Grammar;
  if (grammar.schema !== "vize.dialectInputGrammar" || grammar.version !== 1) {
    throw new Error("unsupported dialect input grammar");
  }
  const seen = new Set<string>();
  const files = new Map<string, string>();
  const cases = grammar.cases.map((row) => {
    if (!/^[a-z0-9-]+$/.test(row.id) || seen.has(row.id)) {
      throw new Error(`invalid or duplicate dialect input ID: ${row.id}`);
    }
    seen.add(row.id);
    validateSelector(row);
    validateWitness(row.witness);
    const render = productions[row.production];
    if (render == null) throw new Error(`unknown grammar production: ${row.production}`);
    validateTokens(row);
    const source = render(row.tokens);
    const file = `generated/${row.id}.html`;
    files.set(file, source);
    return {
      id: `dialect/${row.id}`,
      dialects: row.dialects,
      selector: row.selector,
      expectedParserMode: row.expectedParserMode,
      sourceRule: row.production,
      witness: row.witness,
      input: { path: file, sha256: createHash("sha256").update(source).digest("hex") },
      state: "input-only",
    };
  });
  return {
    inventory: { schema: "vize.dialectInputInventory", version: 1, cases },
    files,
  };
}

function validateTokens(row: GrammarRow): void {
  if (row.production === "list") {
    const expected =
      row.selector.vueVersion === "1"
        ? { loop: 'v-for="item in items"', event: '@click="select"' }
        : { loop: 'v-repeat="item: items"', event: 'v-on="click: select"' };
    if (JSON.stringify(row.tokens) !== JSON.stringify(expected)) {
      throw new Error(`list production tokens drifted: ${row.id}`);
    }
  } else if (JSON.stringify(row.tokens) !== "{}") {
    throw new Error(`unexpected tokens for ${row.production}: ${row.id}`);
  }
}

function validateSelector(row: GrammarRow): void {
  const { vueVersion, templateSyntax } = row.selector;
  const selectors: Readonly<Record<string, readonly [string, string]>> = {
    "0.10": ["vue0-template", "legacy-v0.10"],
    "0.11": ["vue0-template", "legacy-v0.11"],
    "1": ["vue1-template", "legacy-v1"],
    "3": ["vue-quirks", "quirks"],
  };
  const expected = selectors[vueVersion];
  if (
    expected == null ||
    !["standard", "quirks"].includes(templateSyntax) ||
    (templateSyntax === "quirks") !== (vueVersion === "3") ||
    JSON.stringify(row.dialects) !== JSON.stringify([expected[0]]) ||
    row.expectedParserMode !== expected[1]
  ) {
    throw new Error(`dialect selector is inconsistent: ${row.id}`);
  }
}

function validateWitness(witness: Partial<Witness> | undefined): void {
  if (
    typeof witness?.file !== "string" ||
    typeof witness?.selector !== "string" ||
    path.isAbsolute(witness.file) ||
    witness.file.split(/[\\/]/).includes("..")
  ) {
    throw new Error("invalid dialect grammar witness");
  }
  const file = path.join(root, witness.file);
  const source = fs.readFileSync(file, "utf8");
  if (source.split(String(witness.selector)).length !== 2) {
    throw new Error(`stale dialect grammar witness: ${witness.file}`);
  }
}

const byCodeUnit = (left: string, right: string): number =>
  left < right ? -1 : left > right ? 1 : 0;

export function checkDialectInputs() {
  const { inventory, files } = buildDialectInputs();
  const expectedInventory = formatInventory(inventory);
  if (fs.readFileSync(inventoryPath, "utf8") !== expectedInventory) {
    throw new Error("dialect input inventory drifted; regenerate it");
  }
  const actualNames = fs.readdirSync(generatedDir).sort(byCodeUnit);
  const expectedNames = [...files.keys()].map((file) => path.basename(file)).sort(byCodeUnit);
  if (JSON.stringify(actualNames) !== JSON.stringify(expectedNames)) {
    throw new Error("dialect input file set drifted");
  }
  for (const [file, source] of files) {
    if (fs.readFileSync(path.join(root, corpusDir, file), "utf8") !== source) {
      throw new Error(`dialect input bytes drifted: ${file}`);
    }
  }
}

function formatInventory(inventory: unknown): string {
  // Match the repository formatter's compact one-label JSON arrays while
  // keeping generation dependency-free for the fixture check.
  return `${JSON.stringify(inventory, null, 2).replace(/"dialects": \[\n\s+("[^"]+")\n\s+\]/g, '"dialects": [$1]')}\n`;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) {
  if (process.argv[2] === "--write") {
    const { inventory, files } = buildDialectInputs();
    fs.mkdirSync(generatedDir, { recursive: true });
    for (const [file, source] of files) fs.writeFileSync(path.join(root, corpusDir, file), source);
    fs.writeFileSync(inventoryPath, formatInventory(inventory));
  } else if (process.argv[2] === "--check") {
    checkDialectInputs();
  } else {
    throw new Error("usage: generate-dialect-inputs.ts --check|--write");
  }
}
