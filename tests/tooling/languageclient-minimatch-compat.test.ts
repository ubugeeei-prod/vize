import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { compileFunction } from "node:vm";
import YAML from "yaml";

const here = path.dirname(fileURLToPath(import.meta.url));
const privateRoot = process.env.VIZE_COMPAT_ROOT;
assert.ok(
  !process.env.GITHUB_ACTIONS || !privateRoot,
  "Actions must exercise its real frozen checkout",
);
const root = fs.realpathSync(privateRoot ?? path.resolve(here, "../.."));
const patchHash = "507db4cd05afc3ee9f45acce0e15a195181d015379bff1e09a919a67b62bc41c";
const patchPath = "patches/vscode-languageclient@9.0.1.patch";
const baselineHash = "624155729411c8abfa3651f9b97102228654c7513e575d096c7885dc79d405ed";
const clientSRI =
  "sha512-JZiimVdvimEuHh5olxhxkht09m3JzUGwggb5eRUkzzJhZ2KjCN0nh55VfiED9oez9DyF8/fz1g1iBV3h+0Z2EA==";
const matcherSRI =
  "sha512-t+Vn6DX4wPCxS23y03C0uhF7S1jpi1GfqPrnlEipCJaA2ykiNbMkVlDutZtmvLfRcUN6R8AYVlzTsQrT4RZoLQ==";
const hashes: Record<string, string> = {
  "fileOperations.js": "182d6460a8f7273a770acdce7125ea96dd7ad5fbb2d8dc8040f82520b560de4f",
  "notebook.js": "a756d424290e09633dd94ec4229b0c07c50e4b94d78981f6f85c312af46106ed",
  "diagnostic.js": "42fe8728c31ff261dfa58ed7f92fa5ce611c6ddf37db97600755fec4c6804d62",
};
const sha = (bytes: Uint8Array | string) => createHash("sha256").update(bytes).digest("hex");
function record(value: unknown): Record<string, unknown> {
  assert.ok(value && typeof value === "object" && !Array.isArray(value), "expected record");
  return value as Record<string, unknown>;
}
const read = (filename: string) => record(JSON.parse(fs.readFileSync(filename, "utf8")));
const require = createRequire(path.join(root, "package.json"));
const clientRoot = fs.realpathSync(
  path.dirname(require.resolve("vscode-languageclient/package.json")),
);
const client = createRequire(path.join(clientRoot, "package.json"));
assert.equal(read(path.join(clientRoot, "package.json")).version, "9.0.1");
const matcherRoot = fs.realpathSync(path.dirname(client.resolve("minimatch/package.json")));
assert.equal(read(path.join(matcherRoot, "package.json")).version, "11.0.1");
type Matcher = { set: readonly unknown[]; match(value: string): boolean };
type MatcherAPI = {
  Minimatch: new (pattern: unknown, options?: Record<string, unknown>) => Matcher;
};
const apiValue: unknown = client("minimatch");
assert.ok(apiValue && typeof apiValue === "object" && "Minimatch" in apiValue);
const minimatch = apiValue as MatcherAPI;
assert.equal(typeof minimatch.Minimatch, "function");
assert.ok(!("makeRe" in minimatch.Minimatch.prototype));
assert.ok(!("makeRe" in new minimatch.Minimatch("**/*.vue")));
const protocol: unknown = client("vscode-languageserver-protocol");
const workspaceBytes = fs.readFileSync(path.join(root, "pnpm-workspace.yaml"));
const lockBytes = fs.readFileSync(path.join(root, "pnpm-lock.yaml"));
const workspaceValue: unknown = YAML.parse(workspaceBytes.toString("utf8"));
const workspace = record(workspaceValue);
assert.equal(record(workspace.patchedDependencies)["vscode-languageclient@9.0.1"], patchPath);
assert.equal(record(workspace.overrides).minimatch, "11.0.1");
assert.equal(sha(fs.readFileSync(path.join(root, patchPath))), patchHash);
const documents = YAML.parseAllDocuments(lockBytes.toString("utf8"));
assert.equal(documents.length, 2);
assert.ok(documents.every((document) => document.errors.length === 0));
const lockValue: unknown = documents[1].toJS();
const lock = record(lockValue);
const lockedPatch = record(lock.patchedDependencies)["vscode-languageclient@9.0.1"];
if (typeof lockedPatch === "string") assert.equal(lockedPatch, patchHash);
else assert.deepEqual(lockedPatch, { hash: patchHash, path: patchPath });
const packages = record(lock.packages);
assert.deepEqual(record(packages["vscode-languageclient@9.0.1"]).resolution, {
  integrity: clientSRI,
});
assert.deepEqual(record(packages["minimatch@11.0.1"]).resolution, { integrity: matcherSRI });
const snapshots = record(lock.snapshots);
const clientKeys = Object.keys(snapshots).filter((key) => key.startsWith("vscode-languageclient@"));
assert.deepEqual(clientKeys, ["vscode-languageclient@9.0.1(patch_hash=" + patchHash + ")"]);
assert.equal(record(record(snapshots[clientKeys[0]]).dependencies).minimatch, "11.0.1");
const source: Record<string, string> = {};
for (const [file, expected] of Object.entries(hashes)) {
  source[file] = fs.readFileSync(path.join(clientRoot, "lib/common", file), "utf8");
  assert.equal(sha(source[file]), expected, "approved installed client source: " + file);
}
function between(value: string, start: string, end: string): string {
  const first = value.indexOf(start);
  assert.ok(first >= 0 && value.indexOf(start, first + 1) === -1, "one exact source route");
  const last = value.indexOf(end, first + start.length);
  assert.ok(last > first);
  return value.slice(first, last);
}
// Execute the installed bodies, rather than reproducing their glob predicates.
// These finite controls make no full VSCode host or file-kind filesystem claim.
const fileBody = between(
  source["fileOperations.js"],
  "class FileOperationFeature {",
  "class NotificationFileOperationFeature",
);
const notebookBody = between(
  source["notebook.js"],
  "    function matchNotebook(filter, notebookDocument) {",
  "    $NotebookDocumentFilter.matchNotebook = matchNotebook;",
);
const diagnosticBody = between(
  source["diagnostic.js"],
  "        const matchResource = (resource) => {",
  "        const matches = (document) => {",
);
type Uri = { fsPath: string; scheme: string };
type Item = { id: number; uri: Uri };
type FileRoute = {
  register(value: unknown): void;
  filter(value: { files: Item[] }, prop: (item: Item) => Uri): Promise<{ files: Item[] }>;
};
type FileClass = new (
  client: unknown,
  event: (handler: unknown, receiver: unknown) => { dispose(): void },
  registration: unknown,
  capability: string,
  server: string,
) => FileRoute;
type Notebook = (filter: unknown, document: { notebookType: string; uri: Uri }) => boolean;
type Diagnostic = (resource: Uri) => boolean;
const FileRoute = compileFunction("return " + fileBody, ["minimatch", "proto"])(
  minimatch,
  protocol,
) as unknown as FileClass;
const notebook = compileFunction("return " + notebookBody.trim(), ["minimatch"])(
  minimatch,
) as unknown as Notebook;
const makeDiagnostic = compileFunction(diagnosticBody + "\nreturn matchResource;", [
  "minimatch",
  "options",
  "diagnosticPullOptions",
  "vscode_languageserver_protocol_1",
]);
function diagnostic(
  selector: unknown[],
  options: { match?: (selector: unknown[], uri: Uri) => boolean } = {},
): Diagnostic {
  return makeDiagnostic(
    minimatch,
    { documentSelector: selector },
    options,
    protocol,
  ) as unknown as Diagnostic;
}
type Vector = {
  id: string;
  pattern: unknown;
  paths: string[];
  ignoreCase?: boolean;
  filterScheme?: string;
};
type Result = { value: boolean } | { error: string; message: string };
function failure(value: unknown): { error: string; message: string } {
  assert.ok(value && typeof value === "object" && "name" in value && "message" in value);
  return { error: String(value.name), message: String(value.message) };
}
async function evaluate(vector: Vector) {
  let subscriptions = 0;
  const route = new FileRoute(
    {},
    () => {
      subscriptions++;
      return { dispose() {} };
    },
    { method: "workspace/didCreateFiles" },
    "didCreate",
    "didCreate",
  );
  let file: { selected: number[]; subscriptions: number } | { error: string; message: string };
  try {
    route.register({
      id: vector.id,
      registerOptions: {
        filters: [
          {
            scheme: vector.filterScheme,
            pattern: { glob: vector.pattern, options: { ignoreCase: vector.ignoreCase } },
          },
        ],
      },
    });
    const filtered = await route.filter(
      { files: vector.paths.map((fsPath, id) => ({ id, uri: { fsPath, scheme: "file" } })) },
      (item) => item.uri,
    );
    file = { selected: filtered.files.map((item) => item.id), subscriptions };
  } catch (error) {
    file = failure(error);
  }
  const notebooks: Result[] = [];
  const diagnostics: Result[] = [];
  for (const fsPath of vector.paths) {
    const uri = { fsPath, scheme: "file" };
    try {
      notebooks.push({
        value: notebook(
          { pattern: vector.pattern, scheme: vector.filterScheme },
          { notebookType: "jupyter", uri },
        ),
      });
    } catch (error) {
      notebooks.push(failure(error));
    }
    try {
      diagnostics.push({
        value: diagnostic([{ pattern: vector.pattern, scheme: vector.filterScheme }])(uri),
      });
    } catch (error) {
      diagnostics.push(failure(error));
    }
  }
  return { file, notebook: notebooks, diagnostic: diagnostics };
}
const vectors: Vector[] = [
  {
    id: "vue-dot-and-nonmatch",
    pattern: "**/*.vue",
    paths: ["/repo/App.vue", "/repo/App.ts", "/repo/.hidden.vue"],
  },
  {
    id: "ignore-case-file-operations",
    pattern: "**/APP.VUE",
    ignoreCase: true,
    paths: ["/repo/App.vue", "/repo/App.ts"],
  },
  {
    id: "extglob-options-noext",
    pattern: "**/@(App|Other).vue",
    paths: ["/repo/App.vue", "/repo/Other.vue", "/repo/@(App|Other).vue"],
  },
  {
    id: "windows-path-normalization",
    pattern: "C:/**/*.vue",
    paths: ["C:\\repo\\App.vue", "C:\\repo\\App.ts"],
  },
  { id: "scheme-refusal", pattern: "**/*.vue", filterScheme: "untitled", paths: ["/repo/App.vue"] },
  { id: "empty-pattern-refusal", pattern: "", paths: ["", "/repo/App.vue"] },
  { id: "comment-pattern-refusal", pattern: "#*.vue", paths: ["#x.vue", "/repo/App.vue"] },
  {
    id: "unterminated-class-is-literal",
    pattern: "**/[broken.vue",
    paths: ["/repo/[broken.vue", "/repo/broken.vue"],
  },
  { id: "unterminated-brace-is-literal", pattern: "{foo,bar", paths: ["{foo,bar", "foo"] },
  { id: "malformed-number", pattern: 42, paths: ["/repo/App.vue"] },
  { id: "malformed-null", pattern: null, paths: ["/repo/App.vue"] },
  { id: "pattern-length-limit", pattern: "x".repeat(65537), paths: ["x"] },
];
const securityVectors: Vector[] = [
  {
    id: "many-star-suffix-miss-hit",
    pattern: "a*".repeat(64) + "Z",
    paths: ["a".repeat(1024), "a".repeat(1024) + "Z"],
  },
  {
    id: "many-star-prefix-miss-hit",
    pattern: "*a".repeat(64) + "Z",
    paths: ["a".repeat(1024), "a".repeat(1024) + "Z"],
  },
  {
    id: "repeated-globstar-path-miss-hit",
    pattern: "**/".repeat(64) + "Z",
    paths: ["a/".repeat(128) + "X", "a/".repeat(128) + "Z"],
  },
  {
    id: "long-literal-plus-wildcard-miss-hit",
    pattern: "a".repeat(16000) + "?",
    paths: ["a".repeat(16000), "a".repeat(16000) + "X"],
  },
];
if (process.argv[2] === "--languageclient-security") {
  const index = Number(process.argv[3]);
  assert.ok(Number.isSafeInteger(index) && index >= 0 && index < securityVectors.length);
  const actual = await evaluate(securityVectors[index]);
  assert.deepEqual(actual, {
    file: { selected: [1], subscriptions: 1 },
    notebook: [{ value: false }, { value: true }],
    diagnostic: [{ value: false }, { value: true }],
  });
  process.stdout.write(
    JSON.stringify({ id: securityVectors[index].id, node: process.version, actual }) + "\n",
  );
} else {
  const { test } = await import("node:test");
  test("the frozen installed client and Min11 retain approved source/lock bindings", () => {
    let sourceSha: string | null = null;
    if (!privateRoot) {
      const git = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
      assert.equal(git.status, 0, git.stderr);
      sourceSha = git.stdout.trim();
      if (process.env.GITHUB_SHA) assert.equal(sourceSha, process.env.GITHUB_SHA);
    }
    console.log(
      JSON.stringify({
        schema: "vize.languageclient-minimatch-installed",
        mode: privateRoot ? "explicit-private-root" : "installed-checkout",
        sourceSha,
        node: process.version,
        root,
        clientRoot,
        matcherRoot,
        patchHash,
        workspaceSha256: sha(workspaceBytes),
        lockSha256: sha(lockBytes),
        installedSourceHashes: hashes,
        matcherEntrypointSha256: sha(fs.readFileSync(client.resolve("minimatch"))),
        scope:
          "Actual installed body extraction; no full VSCode host or file-kind filesystem branch claim",
      }),
    );
  });
  test("all twelve complete installed route/error vectors retain their recorded baseline", async () => {
    const bytes = fs.readFileSync(
      path.join(here, "fixtures/languageclient-minimatch-baseline.json"),
    );
    assert.equal(sha(bytes), baselineHash);
    const baseline = record(JSON.parse(bytes.toString("utf8")));
    assert.equal(baseline.schema, "vize.languageclient-minimatch-finite-baseline");
    assert.ok(Array.isArray(baseline.cases));
    const cases = baseline.cases.map(record);
    assert.deepEqual(
      cases.map((item) => item.id),
      vectors.map((item) => item.id),
    );
    const outcomes: unknown[] = [];
    for (let index = 0; index < vectors.length; index++) {
      const actual = await evaluate(vectors[index]);
      assert.deepEqual(actual, cases[index].expected, vectors[index].id);
      outcomes.push({ id: vectors[index].id, actual });
    }
    const uri = { fsPath: "/repo/App.vue", scheme: "file" };
    assert.equal(notebook({ notebookType: "python" }, { notebookType: "jupyter", uri }), false);
    assert.equal(notebook("jupyter", { notebookType: "jupyter", uri }), true);
    assert.equal(diagnostic([{ language: "typescript", pattern: "**/*.vue" }])(uri), false);
    assert.equal(diagnostic([{ pattern: "#comment" }], { match: () => true })(uri), true);
    console.log(JSON.stringify({ baselineSha256: baselineHash, outcomes }));
  });
  test(
    "all twenty-four hostile wildcard route outcomes use the real Min11 matcher",
    { timeout: 60_000 },
    () => {
      const runs: unknown[] = [];
      for (let index = 0; index < securityVectors.length; index++) {
        const result = spawnSync(
          process.execPath,
          [fileURLToPath(import.meta.url), "--languageclient-security", String(index)],
          { cwd: root, encoding: "utf8", timeout: 10_000, maxBuffer: 1024 * 1024 },
        );
        assert.equal(result.status, 0, result.stdout + result.stderr);
        const actual = record(JSON.parse(result.stdout));
        assert.equal(actual.id, securityVectors[index].id);
        assert.equal(actual.node, process.version);
        runs.push(actual);
      }
      assert.equal(sha(fs.readFileSync(path.join(root, "pnpm-lock.yaml"))), sha(lockBytes));
      console.log(
        JSON.stringify({
          cases: 24,
          runs,
          scope: "Finite completion/match laws, not a performance or universal complexity claim",
        }),
      );
    },
  );
}
