// Parser differential oracle for the HTML content-model checker (P4-11a).
//
// The checker claims tier `exact` for its parser family: in a no-quirks
// document body, a statically known element chain is either inserted
// faithfully by the HTML parser or it is not, and the checker must name the
// node that diverges first — never "unknown". The checker's verdicts over the
// differential universe are committed by the Rust test
// `crates/vize_patina/tests/html_content_model_differential.rs` as
// `crates/vize_patina/tests/html_content_model/checker-verdicts.tsv`; this
// script recomputes every verdict with an independent HTML parser and requires
// exact agreement.
//
// Engines:
//   --engine chromium  Chromium's parser (DOMParser, text/html) through
//                      Playwright — the reference: it builds the DOM hydration
//                      compares against, including the 2025 customizable-select
//                      parser changes the pinned WHATWG snapshot describes.
//   --engine parse5    parse5 (html5lib-tests conformant) for sandboxes without
//                      a browser; it predates the customizable-select parser, so
//                      chains through `select`/`option`/`optgroup` are skipped
//                      and counted, never compared.
//
// A chain is serialized the way Vue's SSR renderer emits it (`<a><b></b></a>`,
// void elements without end tags, `#text` as `x`); its verdict is the length
// of the shortest prefix whose parsed DOM is not the authored single-child
// chain (0 when the whole chain is faithful).
//
// Usage:
//   node tools/support/compat/davinci/html-content-model-oracle.mjs --engine chromium|parse5 [--module <path>]

import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..");
const verdictsPath = path.join(
  repoRoot,
  "crates/vize_patina/tests/html_content_model/checker-verdicts.tsv",
);
const VOID = new Set([
  "area",
  "base",
  "br",
  "col",
  "embed",
  "hr",
  "img",
  "input",
  "link",
  "meta",
  "param",
  "source",
  "track",
  "wbr",
]);
const SELECT_FAMILY = new Set(["select", "option", "optgroup", "datalist", "selectedcontent"]);

function arg(name) {
  const index = process.argv.indexOf(name);
  return index === -1 ? undefined : process.argv[index + 1];
}

function serialize(tags) {
  let open = "";
  let close = "";
  for (const tag of tags) {
    if (tag === "#text") {
      open += "x";
      continue;
    }
    open += `<${tag}>`;
    close = (VOID.has(tag) ? "" : `</${tag}>`) + close;
  }
  return open + close;
}

function document(tags) {
  return `<!DOCTYPE html><html><head></head><body>${serialize(tags)}</body></html>`;
}

function readChains() {
  const chains = [];
  const universes = new Map();
  for (const line of readFileSync(verdictsPath, "utf8").split("\n")) {
    if (!line || line.startsWith("#")) continue;
    const [kind, section, ...rest] = line.split("\t");
    if (kind === "universe") universes.set(section, rest[0].split(" "));
    if (kind === "chain") {
      const prefix = rest[0].split(" ");
      const digits = rest[1];
      universes.get(section).forEach((leaf, index) => {
        chains.push({ tags: [...prefix, leaf], expected: digits[index] });
      });
    }
  }
  return chains;
}

// parse5 tree walk: the faithful-chain predicate over its default tree adapter.
function parse5Faithful(parse5, tags) {
  const doc = parse5.parse(document(tags));
  const html = doc.childNodes.find((node) => node.nodeName === "html");
  let node = html.childNodes.find((child) => child.nodeName === "body");
  for (const tag of tags) {
    const kids = node.childNodes;
    if (kids.length !== 1) return false;
    const kid = kids[0];
    const ok =
      tag === "#text"
        ? kid.nodeName === "#text" && kid.value === "x"
        : kid.tagName !== undefined && kid.tagName.toLowerCase() === tag.toLowerCase();
    if (!ok) return false;
    node = kid;
  }
  return (node.childNodes ?? []).length === 0;
}

function verdictWith(faithful, memo, tags) {
  for (let k = 1; k <= tags.length; k++) {
    const key = tags.slice(0, k).join(" ");
    if (!memo.has(key)) memo.set(key, faithful(tags.slice(0, k)));
    if (!memo.get(key)) return String(k);
  }
  return "0";
}

async function chromiumVerdicts(modulePath, chains) {
  const { chromium } = await import(pathToFileURL(modulePath).href);
  const browser = await chromium.launch();
  const page = await browser.newPage();
  const documents = chains.map((chain) => chain.tags);
  const verdicts = await page.evaluate(
    ({ documents, voidTags }) => {
      const VOID = new Set(voidTags);
      const parser = new DOMParser();
      const serialize = (tags) => {
        let open = "";
        let close = "";
        for (const tag of tags) {
          if (tag === "#text") {
            open += "x";
            continue;
          }
          open += `<${tag}>`;
          close = (VOID.has(tag) ? "" : `</${tag}>`) + close;
        }
        return open + close;
      };
      const faithful = (tags) => {
        const doc = parser.parseFromString(
          `<!DOCTYPE html><html><head></head><body>${serialize(tags)}</body></html>`,
          "text/html",
        );
        let node = doc.body;
        for (const tag of tags) {
          const kids = node.childNodes;
          if (kids.length !== 1) return false;
          const kid = kids[0];
          const ok =
            tag === "#text"
              ? kid.nodeType === 3 && kid.data === "x"
              : kid.nodeType === 1 && kid.localName.toLowerCase() === tag.toLowerCase();
          if (!ok) return false;
          node = kid;
        }
        return (node.childNodes ?? []).length === 0;
      };
      const memo = new Map();
      return documents.map((tags) => {
        for (let k = 1; k <= tags.length; k++) {
          const key = tags.slice(0, k).join(" ");
          if (!memo.has(key)) memo.set(key, faithful(tags.slice(0, k)));
          if (!memo.get(key)) return String(k);
        }
        return "0";
      });
    },
    { documents, voidTags: [...VOID] },
  );
  const version = browser.version();
  await browser.close();
  return { verdicts, label: `chromium ${version}` };
}

const engine = arg("--engine");
const chains = readChains();
let result;
let compared = chains;
let skipped = 0;
if (engine === "chromium") {
  const modulePath =
    arg("--module") ?? path.join(repoRoot, "tests/node_modules/@playwright/test/index.mjs");
  result = await chromiumVerdicts(modulePath, chains);
} else if (engine === "parse5") {
  const modulePath =
    arg("--module") ?? path.join(repoRoot, "node_modules/.pnpm/node_modules/parse5/dist/index.js");
  const parse5 = await import(pathToFileURL(modulePath).href);
  compared = chains.filter((chain) => !chain.tags.some((tag) => SELECT_FAMILY.has(tag)));
  skipped = chains.length - compared.length;
  const memo = new Map();
  result = {
    verdicts: compared.map((chain) =>
      verdictWith((tags) => parse5Faithful(parse5, tags), memo, chain.tags),
    ),
    label: "parse5",
  };
} else {
  console.error("usage: html-content-model-oracle.mjs --engine chromium|parse5 [--module <path>]");
  process.exit(2);
}

const mismatches = [];
compared.forEach((chain, index) => {
  if (result.verdicts[index] !== chain.expected) {
    mismatches.push(
      `${chain.tags.join(" > ")}: checker ${chain.expected}, ${result.label} ${result.verdicts[index]}`,
    );
  }
});
console.log(
  `html-content-model-oracle: ${result.label} compared=${compared.length} skipped=${skipped} mismatches=${mismatches.length}`,
);
for (const line of mismatches) console.log(`MISMATCH ${line}`);
process.exit(mismatches.length === 0 ? 0 : 1);
