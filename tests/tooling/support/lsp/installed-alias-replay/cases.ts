import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { gunzipSync } from "node:zlib";

type RecordValue = Record<string, unknown>;
export type AliasForm = "Plain" | "Defaults" | "Partial";
export type AliasName = "hidden" | "tone";
export type DefinitionQuery = { name: AliasName; query: string; definition: unknown };
type CommonCase = {
  id: string;
  captureFile: string;
  context: string;
  form: AliasForm;
  newline: "\n" | "\r\n";
  input: string;
  authoredUri: string;
  expected: Readonly<RecordValue>;
};
export type FrozenAliasCase = CommonCase &
  (
    | { kind: "transaction"; name: AliasName; query: string; newName: string; golden: string }
    | { kind: "definitions"; definitionQueries: readonly DefinitionQuery[] }
  );
export type FrozenPacketReceipt = {
  file: string;
  gzipSha256: string;
  rawPacketSha256: string;
  context: string;
};

// Independently authored source bytes, never derived from captured provider responses.
export const AUTHORED_SOURCE_SHAS = Object.freeze({
  "E.vue.txt": "7a32f47587a687f195b78597ffec552480594c928b6efc675715253a99413a36",
  "supplemental/E-defaults.vue.txt":
    "d1bf2baf6180e1e8fb94108e5204c5bb38d4cbfa343f74c8eb89209ac65b0a6a",
  "supplemental/E-partial.vue.txt":
    "889d4ce11469e4515428f9b062f2f87847535d684c47af9ceff6e1d76d5afb07",
  "supplemental/E-plain-hidden.vue.txt":
    "68a062ba4296e2180d9968358d26d403507560c8e244b01375ae4757f7855187",
  "supplemental/E-plain-tone.vue.txt":
    "d6fc5c63a5b77d6d4b15fe0b81025706d13abc201046f8e480428b2cbf78c12a",
  "supplemental/E-defaults-hidden.vue.txt":
    "bae35436eebe79b337785fc6eca02430d599696c0874eb6e04d88f0a66fea793",
  "supplemental/E-defaults-tone.vue.txt":
    "fb4fffa0b6628f1de42c697d5f3889ecf09852d226ec151e81637e7912bb9df8",
  "supplemental/E-partial-hidden.vue.txt":
    "0793c2666df8aa430b3217a9e6922d20dadcf85df74a9ea119dd73327820b61f",
  "supplemental/E-partial-tone.vue.txt":
    "e3efb49eac60b035abd81b26e88e8063d99010d7f95ecd716c74fdaa5f661b0d",
});
const RECEIPT_SHA = "e1de0ce0a7e62730b3fc8f8a5969d8831697af164750859e31eb750de3518d32";
const forms: AliasForm[] = ["Plain", "Defaults", "Partial"];
const names: AliasName[] = ["hidden", "tone"];
const newlines = ["\n", "\r\n"] as const;
const sha = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const uriLike = /^[A-Za-z][A-Za-z0-9+.-]*:\S/;
const object = (value: unknown): RecordValue => {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "expected object");
  return value as RecordValue;
};
function freeze<T>(value: T): T {
  if (value && typeof value === "object") {
    for (const item of Object.values(value)) freeze(item);
    Object.freeze(value);
  }
  return value;
}
function needles(form: AliasForm, name: AliasName): string[] {
  if (name === "hidden") return ["hidden?:", 'hidden"'];
  return form === "Defaults" ? ["tone?:", 'tone: "light"', 'tone"'] : ["tone?:", 'tone"'];
}
function context(form: AliasForm, newline: string, name?: AliasName, query?: string): string {
  return `${name ? `alias ${form}, ${name}, ${query}` : `alias definitions ${form}`}, newline=${JSON.stringify(newline)}`;
}

/** Both byte identities are checked before JSON parsing; actual is never inspected. */
export function decodeFrozenExpected(gzip: Uint8Array, receipt: FrozenPacketReceipt) {
  assert.equal(sha(gzip), receipt.gzipSha256, "gzip SHA custody");
  const raw = gunzipSync(gzip);
  assert.equal(sha(raw), receipt.rawPacketSha256, "raw packet SHA custody");
  const packet = object(JSON.parse(new TextDecoder("utf8", { fatal: true }).decode(raw)));
  assert.equal(packet.context, receipt.context, "context receipt custody");
  return {
    context: receipt.context,
    initialFiles: packet.initialFiles,
    expected: object(packet.expected),
  };
}

function rebind(value: unknown, oldUri: string, newUri: string, field = ""): unknown {
  if (typeof value === "string") {
    if (field === "uri" || uriLike.test(value)) {
      assert.equal(value, oldUri, "unknown authored URI");
      return newUri;
    }
    return value;
  }
  if (Array.isArray(value)) return value.map((item) => rebind(item, oldUri, newUri));
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value).map(([key, item]) => {
      const uriKey = field === "changes" || uriLike.test(key);
      if (uriKey) assert.equal(key, oldUri, "unknown authored URI key");
      return [uriKey ? newUri : key, rebind(item, oldUri, newUri, key)];
    }),
  );
}

/** Rebinding affects only frozen expectations and requires one explicit authored identity. */
export function rebindFrozenExpected(
  testCase: FrozenAliasCase,
  binding: { oldUri: string; newUri: string },
): Readonly<RecordValue> {
  assert.equal(binding.oldUri, testCase.authoredUri, "authored URI binding mismatch");
  assert.ok(
    binding.newUri.startsWith("file:///") && binding.newUri.endsWith("/E.vue"),
    "new E URI must be explicit",
  );
  return freeze(object(rebind(testCase.expected, binding.oldUri, binding.newUri)));
}

export function loadFrozenAliasCases(fixtureRoot: string): readonly FrozenAliasCase[] {
  const sources = new Map<string, string>();
  for (const [file, expectedSha] of Object.entries(AUTHORED_SOURCE_SHAS)) {
    const bytes = readFileSync(path.join(fixtureRoot, file));
    assert.equal(sha(bytes), expectedSha, `independent source SHA custody: ${file}`);
    sources.set(file, new TextDecoder("utf8", { fatal: true }).decode(bytes));
  }
  const historical = path.join(fixtureRoot, "supplemental/historical-c2bc");
  const receiptBytes = readFileSync(path.join(historical, "receipt.json.txt"));
  assert.equal(sha(receiptBytes), RECEIPT_SHA, "frozen receipt SHA custody");
  const receipt = object(JSON.parse(receiptBytes.toString("utf8")));
  assert.ok(Array.isArray(receipt.packets));
  assert.equal(receipt.packets.length, 32, "exact frozen case count");
  const wanted = new Set<string>();
  for (const form of forms)
    for (const newline of newlines) {
      wanted.add(context(form, newline));
      for (const name of names)
        for (const query of needles(form, name)) wanted.add(context(form, newline, name, query));
    }
  const result: FrozenAliasCase[] = [];
  for (const row of receipt.packets) {
    const entry = object(row) as FrozenPacketReceipt;
    assert.match(entry.file, /^lsp-vue-[A-Za-z0-9]+\.json\.txt\.gz$/, "closed capture filename");
    const packet = decodeFrozenExpected(readFileSync(path.join(historical, entry.file)), entry);
    assert.ok(
      wanted.delete(packet.context),
      `unknown or duplicate frozen context: ${packet.context}`,
    );
    const suffix = /, newline=("(?:\\r)?\\n")$/.exec(packet.context);
    assert.ok(suffix, "closed newline context");
    const newline = JSON.parse(suffix[1]) as "\n" | "\r\n";
    const header = packet.context.slice(0, -suffix[0].length);
    const match =
      /^alias (?:(definitions) )?(Plain|Defaults|Partial)(?:, (hidden|tone), (.+))?$/.exec(header);
    assert.ok(match, "closed alias context");
    const form = match[2] as AliasForm;
    const sourceFile =
      form === "Plain" ? "E.vue.txt" : `supplemental/E-${form.toLowerCase()}.vue.txt`;
    const input = sources.get(sourceFile)!.replaceAll("\n", newline);
    assert.deepEqual(
      packet.initialFiles,
      [{ file: "E.vue", text: input }],
      "independent initial source custody",
    );
    const id = entry.file.slice(0, -".json.txt.gz".length);
    const authoredUri = `file:///home/runner/_work/vize/vize/target/vize-tests/tests/${id}/E.vue`;
    rebind(packet.expected, authoredUri, authoredUri);
    const common = {
      id,
      captureFile: entry.file,
      context: packet.context,
      form,
      newline,
      input,
      authoredUri,
      expected: packet.expected,
    };
    if (match[1]) {
      const queries = packet.expected.definitions as DefinitionQuery[];
      assert.ok(Array.isArray(queries));
      assert.deepEqual(
        queries.map(({ name, query }) => ({ name, query })),
        names.flatMap((name) => needles(form, name).map((query) => ({ name, query }))),
        "complete definition query order",
      );
      result.push({ ...common, kind: "definitions", definitionQueries: queries });
    } else {
      const name = match[3] as AliasName;
      const golden = sources
        .get(`supplemental/E-${form.toLowerCase()}-${name}.vue.txt`)!
        .replaceAll("\n", newline);
      const transaction = object(packet.expected.transaction);
      const file = { file: "E.vue", text: golden, disk: golden, diagnostics: [] };
      assert.deepEqual(transaction.files, [file], "independent transaction golden custody");
      assert.deepEqual(
        transaction.independentRepair,
        [{ ...file, version: 3 }],
        "independent repair custody",
      );
      result.push({
        ...common,
        kind: "transaction",
        name,
        query: match[4],
        newName: name === "hidden" ? "concealed" : "palette",
        golden,
      });
    }
  }
  assert.equal(wanted.size, 0, "complete frozen context matrix");
  assert.equal(result.filter((item) => item.kind === "transaction").length, 26);
  assert.equal(result.filter((item) => item.kind === "definitions").length, 6);
  return freeze(result);
}
