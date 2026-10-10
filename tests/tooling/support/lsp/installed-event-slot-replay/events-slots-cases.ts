import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { utf16Position } from "../installed-alias-replay/apply.ts";

type ObjectValue = Record<string, unknown>;
type Family = "event" | "slot" | "quoted-slot";
type SourceFile = { file: string; text: string; golden: string };
export type EventSlotCase = {
  id: string;
  context: string;
  family: Family;
  origin: string;
  newline: "\n" | "\r\n";
  files: readonly SourceFile[];
  query: { file: string; needle: string; newName: string };
  expected: Readonly<ObjectValue>;
};
const sha = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const authoredRoot = "file:///authored-event-slot/";
const fixturePrefix = "tests/_fixtures/differential/lsp/";
// Originals and independent repair files are immutable, including control owners.
export const eventSlotSourceShas = Object.freeze({
  "event-rename/8010/Toggle.vue.txt":
    "9ee595b05e57189ca7141a486318162358784c2f929d6a2fd944f7013b3da0a8",
  "event-rename/8010/App.vue.txt":
    "4166a47d3a1e433f69690c43cd09a80097cdc10da07f98e32030928b0e450d32",
  "event-rename/8010/UpdatedToggle.vue.txt":
    "d33acfb768e592a0f344867485a963ef4b81bf80b977170739d49cc8b3d88348",
  "event-rename/8010/UpdatedApp.vue.txt":
    "46c5553979a65149f7f7b14c75e35ce0aed220d72f5d2173b9f30d1ac644dae0",
  "slot-rename/8011/original/Card.vue.txt":
    "27a5b1890db798d559d09016d20a67c1219a7c3fde6318c9d5351fba4aecea78",
  "slot-rename/8011/original/CardUser.vue.txt":
    "f1f898c6616f9ce27a634dfa7446bef2218d8b610f49e808f07524ba3b92bf62",
  "slot-rename/8011/original/CardRenamed.vue.txt":
    "4e61dc74e3c1bd7297f131ae8334a8960554c418c87aca75b1f3c198590cc53f",
  "slot-rename/8011/original/CardUserRenamed.vue.txt":
    "39c62b66539f4f520a295e3bc901f0ee893c8a890a228e4b1b4a187fe1d218e8",
  "slot-rename/8011/controls/QuotedCard.vue.txt":
    "5f869d248bf739de059d454e74da42a9e221193170bf44660e574ebed8268f53",
  "slot-rename/8011/controls/QuotedUser.vue.txt":
    "f145586fe76c8af4c0e6a74c930c0894729ba1e63c82e696e47ef5234d8dcf5d",
  "slot-rename/8011/controls/Other.vue.txt":
    "2d72c564ac6a243e2591ef4812675a73362964758268a6d48c12e6ef22e09590",
  "slot-rename/8011/controls/QuotedCardRenamed.vue.txt":
    "949c3c3b6c28fc539e9eb37746cab024b6fdd01a9ad20abe3fb0aea4dcb91189",
  "slot-rename/8011/controls/QuotedUserRenamed.vue.txt":
    "1e1e0d595bf9e95007ab6f75125d7cb9a4e7115ff04da66539359e4a76e32cb6",
});
const originalLawShas = {
  "crates/vize/tests/lsp_original_event_rename_cli.rs":
    "71ea12971aacb34c6b4aa137f8feb5676efd193048de040196132edff1b0845a",
  "crates/vize/tests/lsp_original_slot_rename_cli.rs":
    "8674b90bbeb56c07305ceeeaf77e3b26b3b97056cd7066a5e040651537f6b5e7",
  "crates/vize/tests/support/lsp_authored_rename.rs":
    "74d639dcf3f01840fd01582eeffa264266d7c4a8d67ec37be1a2e106e91c668b",
  "crates/vize/tests/support/lsp_vue_project.rs":
    "352aa0542cbfa01407eded744e207957484fd5ab0c0a4a9736f71c8dd869f168",
};
const wholeExpectedShas: Record<Family, readonly string[]> = {
  event: [
    "ce46e364502cb635ccf61d44ce675d96102bc53292b4640bb9bebced1bb82059",
    "d0dd759af8b65bcb13400ad751d9dd9bfddecbaaf6048aa9113ead9487bcbae9",
  ],
  slot: [
    "05c3e89b68a43e0e131f6204f8fda3cd5c2c2a44962ed2361d395106cfca24c7",
    "5991e5043b657edfdab6e1929a97f4950df41950159caf85bda0f3f58b5692be",
  ],
  "quoted-slot": [
    "5c5aade970ead3ae67dc68ad285d34344208123f78f4a8b503203fd694edfe16",
    "f06cd7366eb96cbaa30e2d6aaadf1e3f3b738d89eadb800c90ffd0615e0258a4",
  ],
};
export const eventSlotTsconfig =
  '{\n            "compilerOptions": { "strict": true, "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler", "noEmit": true },\n            "include": ["*.vue"]\n        }';
export const eventSlotVizeConfig = freeze({
  experimentals: { patternedTemplate: false },
  typeChecker: { checkFallthroughAttrs: false, optionsApi: false },
  lsp: { lint: false, typecheck: true, hover: true, crossFile: true },
});
function freeze<T>(value: T): T {
  if (value && typeof value === "object") {
    Object.values(value).forEach(freeze);
    Object.freeze(value);
  }
  return value;
}
function sorted(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sorted);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value)
        .sort(([a], [b]) => a.localeCompare(b, "en"))
        .map(([key, item]) => [key, sorted(item)]),
    );
  return value;
}
const uri = (root: string, file: string) => pathToFileURL(path.join(root, file)).href;
function range(text: string, needle: string, length: number) {
  const offset = text.indexOf(needle);
  assert.ok(offset >= 0, `original first-occurrence needle: ${needle}`);
  const start = utf16Position(text, offset);
  return { start, end: { line: start.line, character: start.character + length } };
}
/** Same authored sites/order as the frozen Rust laws, before any provider query. */
function authorExpected(family: Family, files: readonly SourceFile[]): ObjectValue {
  const sites =
    family === "event"
      ? ([
          [1, 'change="onChange"', 6],
          [0, "change:", 6],
          [0, 'change", true', 6],
        ] as const)
      : family === "slot"
        ? ([
            [0, "header(props", 6],
            [0, 'header" title', 6],
            [1, 'header="', 6],
          ] as const)
        : ([
            [0, 'item-row":', 8],
            [0, 'item-row" title', 8],
            [1, 'item-row="', 8],
          ] as const);
  const references = sites.map(([index, needle, length]) => ({
    uri: authoredRoot + files[index].file,
    range: range(files[index].text, needle, length),
  }));
  const changes: Record<string, unknown[]> = {};
  for (const [index, needle, length] of sites)
    (changes[authoredRoot + files[index].file] ??= []).push({
      range: range(files[index].text, needle, length),
      newText: family === "event" ? "update" : "heading",
    });
  const goldens = files.map(({ file, golden }) => ({
    file,
    text: golden,
    disk: golden,
    diagnostics: [],
  }));
  return {
    references,
    rename: { changes },
    files: goldens,
    independentRepair: goldens.map((file) => ({ ...file, version: 3 })),
  };
}
/** Reads only immutable source/golden bytes; no captured actual result is an oracle. */
export function loadEventSlotCases(repositoryRoot: string): readonly EventSlotCase[] {
  const sources = new Map<string, string>();
  for (const [file, expectedSha] of Object.entries(eventSlotSourceShas)) {
    const bytes = fs.readFileSync(path.join(repositoryRoot, fixturePrefix, file));
    assert.equal(sha(bytes), expectedSha, `original source/golden SHA: ${file}`);
    sources.set(file, new TextDecoder("utf8", { fatal: true }).decode(bytes));
  }
  for (const [file, expectedSha] of Object.entries(originalLawShas))
    assert.equal(
      sha(fs.readFileSync(path.join(repositoryRoot, file))),
      expectedSha,
      `original law SHA: ${file}`,
    );
  assert.equal(
    sha(eventSlotTsconfig),
    "9745d8b2987e4d3bc6b18aa35f8ca96ec1d3a7578b2815b3bfd9523a0bd75581",
  );
  const cases: EventSlotCase[] = [];
  for (const family of ["event", "slot", "quoted-slot"] as const) {
    const folder =
      family === "event"
        ? "event-rename/8010"
        : `slot-rename/8011/${family === "slot" ? "original" : "controls"}`;
    const entries =
      family === "event"
        ? [
            ["Toggle", "Toggle", "UpdatedToggle"],
            ["App", "App", "UpdatedApp"],
          ]
        : family === "slot"
          ? [
              ["Card", "Card", "CardRenamed"],
              ["CardUser", "CardUser", "CardUserRenamed"],
            ]
          : [
              ["Card", "QuotedCard", "QuotedCardRenamed"],
              ["CardUser", "QuotedUser", "QuotedUserRenamed"],
              ["Other", "Other", "Other"],
            ];
    const origins =
      family === "event"
        ? ["Declaration", "EmitCall", "ParentListener"]
        : ["declaration", "outlet", "consumer"];
    for (const [index, origin] of origins.entries())
      for (const [ending, newline] of (["\n", "\r\n"] as const).entries()) {
        const files = entries.map(([file, source, golden]) => ({
          file: `${file}.vue`,
          text: sources.get(`${folder}/${source}.vue.txt`)!.replaceAll("\n", newline),
          golden: sources.get(`${folder}/${golden}.vue.txt`)!.replaceAll("\n", newline),
        }));
        const needles =
          family === "event"
            ? ["hange:", 'hange", true', 'hange="onChange"']
            : family === "slot"
              ? ["eader(props", 'eader" title', 'eader="']
              : ['tem-row":', 'tem-row" title', 'tem-row="'];
        const context =
          family === "event"
            ? `original event ${origin}, newline=${JSON.stringify(newline)}`
            : `${family === "slot" ? "original #8011 slot" : "authored slot controls"} ${origin} ${JSON.stringify(newline)}`;
        const expected = authorExpected(family, files);
        assert.equal(
          sha(JSON.stringify(sorted(expected))),
          wholeExpectedShas[family][ending],
          "whole independently authored expectation SHA",
        );
        cases.push({
          id: `${family}-${origin.toLowerCase()}-${ending === 0 ? "lf" : "crlf"}`,
          context,
          family,
          origin,
          newline,
          files,
          query: {
            file: files[index === 2 ? 1 : 0].file,
            needle: needles[index],
            newName: family === "event" ? "update" : "heading",
          },
          expected,
        });
      }
  }
  assert.equal(cases.length, 18);
  return freeze(cases);
}
function rebind(value: unknown, bindings: ReadonlyMap<string, string>): unknown {
  if (typeof value === "string" && /^[A-Za-z][A-Za-z0-9+.-]*:\S/.test(value)) {
    assert.ok(bindings.has(value), "only explicitly owned original URIs may be rebound");
    return bindings.get(value)!;
  }
  if (Array.isArray(value)) return value.map((item) => rebind(item, bindings));
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [
        /^[A-Za-z][A-Za-z0-9+.-]*:\S/.test(key) ? (rebind(key, bindings) as string) : key,
        rebind(item, bindings),
      ]),
    );
  return value;
}
export function prepareEventSlotExpectation(
  testCase: EventSlotCase,
  projectRoot: string,
): Readonly<ObjectValue> {
  assert.ok(path.isAbsolute(projectRoot), "absolute original project root");
  const bindings = new Map(
    testCase.files.map(({ file }) => [authoredRoot + file, uri(projectRoot, file)]),
  );
  return freeze(rebind(testCase.expected, bindings) as ObjectValue);
}
