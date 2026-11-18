// Oracle packets for the htmlish divergence ledger
// (docs/davinci/plan/htmlish-divergences.json).
//
//   node tools/commands/davinci/htmlish-oracle.ts [--check]
//
// Reads davinci/vize_l1/tests/fixtures/htmlish/sources.json ({ row: { document?, vue? } })
// and writes one packet per row and dialect to
// davinci/vize_l1/tests/fixtures/htmlish/<row>.<dialect>.json with the tree each
// dialect's oracle builds: parse5 (html5lib-tests conformant WHATWG parsing) for
// `document`, `@vue/compiler-dom` `parse` for `vue`. `--check` regenerates in
// memory and fails when a committed packet differs.
//
// Packet shape (both dialects):
//   elements: [name, parentIndex | null, [[attribute, value | null]]] in document order
//   texts:    [parentIndex | null, text]
//   comments: [parentIndex | null, text]
//   errors:   the oracle's error codes, in order
// Text is each oracle's text data (adjacent text merges, as both oracles do);
// an interpolation is its authored source. Vue attribute values are the
// authored source without quotes. The sources hold no character references, so
// decoded and authored text agree (row `character-references` has its own
// fixtures).
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const fixtures = path.join(repoRoot, "davinci/vize_l1/tests/fixtures/htmlish");
const require = createRequire(import.meta.url);

type Attribute = [string, string | null];
export interface Packet {
  row: string;
  dialect: "document" | "vue";
  source: string;
  oracle: { name: string; version: string };
  elements: [string, number | null, Attribute[]][];
  texts: [number | null, string][];
  comments: [number | null, string][];
  errors: string[];
}

interface Parse5Node {
  nodeName: string;
  tagName?: string;
  attrs?: { name: string; value: string }[];
  value?: string;
  data?: string;
  childNodes?: Parse5Node[];
  content?: Parse5Node;
}

function documentPacket(row: string, source: string): Packet {
  const modulePath = path.join(repoRoot, "node_modules/.pnpm/node_modules/parse5");
  const parse5 = require(modulePath) as {
    parse(html: string, options: { onParseError(error: { code: string }): void }): Parse5Node;
  };
  const version = (require(path.join(modulePath, "package.json")) as { version: string }).version;
  const packet: Packet = {
    row,
    dialect: "document",
    source,
    oracle: { name: "parse5", version },
    elements: [],
    texts: [],
    comments: [],
    errors: [],
  };
  const document = parse5.parse(source, {
    onParseError: (error) => packet.errors.push(error.code),
  });
  const visit = (node: Parse5Node, parent: number | null) => {
    // `template` contents live in a separate fragment, as in the DOM.
    const children = node.content?.childNodes ?? node.childNodes ?? [];
    for (const child of children) {
      if (child.nodeName === "#text") packet.texts.push([parent, child.value ?? ""]);
      else if (child.nodeName === "#comment") packet.comments.push([parent, child.data ?? ""]);
      else if (child.tagName !== undefined) {
        packet.elements.push([
          child.tagName,
          parent,
          (child.attrs ?? []).map((attribute) => [attribute.name, attribute.value]),
        ]);
        visit(child, packet.elements.length - 1);
      }
    }
  };
  visit(document, null);
  return packet;
}

interface VueNode {
  type: number;
  tag?: string;
  props?: VueProp[];
  children?: VueNode[];
  loc: { source: string };
  content?: string | { loc: { source: string } };
}

interface VueProp {
  type: number;
  name: string;
  rawName?: string;
  value?: { loc: { source: string } };
  exp?: { loc: { source: string } };
}

function unquote(source: string): string {
  const quote = source[0];
  return (quote === '"' || quote === "'") && source.endsWith(quote) && source.length >= 2
    ? source.slice(1, -1)
    : source;
}

function vuePacket(row: string, source: string): Packet {
  const modulePath = path.join(repoRoot, "tests/node_modules/@vue/compiler-dom");
  const compiler = require(modulePath) as {
    parse(
      template: string,
      options: { comments: boolean; whitespace: string; onError(error: { code: number }): void },
    ): { children: VueNode[] };
  };
  const version = (require(path.join(modulePath, "package.json")) as { version: string }).version;
  const packet: Packet = {
    row,
    dialect: "vue",
    source,
    oracle: { name: "@vue/compiler-dom", version },
    elements: [],
    texts: [],
    comments: [],
    errors: [],
  };
  const root = compiler.parse(source, {
    comments: true,
    whitespace: "preserve",
    onError: (error) => packet.errors.push(String(error.code)),
  });
  const visit = (children: VueNode[], parent: number | null) => {
    for (const child of children) {
      // 1 element, 2 text, 3 comment, 5 interpolation (text in this shape).
      if (child.type === 1 && child.tag !== undefined) {
        packet.elements.push([
          child.tag,
          parent,
          (child.props ?? []).map((prop): Attribute =>
            prop.type === 6
              ? [prop.name, prop.value ? unquote(prop.value.loc.source) : null]
              : [prop.rawName ?? prop.name, prop.exp ? prop.exp.loc.source : null],
          ),
        ]);
        visit(child.children ?? [], packet.elements.length - 1);
      } else if (child.type === 2) {
        packet.texts.push([parent, typeof child.content === "string" ? child.content : ""]);
      } else if (child.type === 5) {
        packet.texts.push([parent, child.loc.source]);
      } else if (child.type === 3) {
        packet.comments.push([parent, typeof child.content === "string" ? child.content : ""]);
      }
    }
  };
  visit(root.children, null);
  return packet;
}

export function packets(): Packet[] {
  const sources = JSON.parse(readFileSync(path.join(fixtures, "sources.json"), "utf8")) as Record<
    string,
    { document?: string; vue?: string }
  >;
  const out: Packet[] = [];
  for (const [row, dialects] of Object.entries(sources)) {
    if (dialects.document !== undefined) out.push(documentPacket(row, dialects.document));
    if (dialects.vue !== undefined) out.push(vuePacket(row, dialects.vue));
  }
  return out;
}

export const packetPath = (packet: Pick<Packet, "row" | "dialect">) =>
  path.join(fixtures, `${packet.row}.${packet.dialect}.json`);

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  const check = process.argv.includes("--check");
  let stale = 0;
  for (const packet of packets()) {
    const file = packetPath(packet);
    const text = JSON.stringify(packet, null, 2) + "\n";
    if (check) {
      let committed = "";
      try {
        committed = readFileSync(file, "utf8");
      } catch {}
      if (committed !== text) {
        stale++;
        console.error(`stale: ${path.relative(repoRoot, file)}`);
      }
    } else writeFileSync(file, text);
  }
  if (stale) process.exit(1);
}
