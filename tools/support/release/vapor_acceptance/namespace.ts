import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { digest, type Row } from "./fixtures.ts";

export const namespaceDirectory = "vapor-svg-namespace-8624";
export const namespaceFix = "d9ced7f2fa7cf185609579e13791dea0fab1e9c1";
const namespaceHashes: Record<string, string> = {
  "App.vue.txt": "3302b4bdfbe306a97822e360314ea018aca6cb3e95de85426d30d9ec3baf3abe",
  "Child.vue.txt": "7ee80a14ddd4dbd687fb30c137cabee5d6be8416765c4734ff6b7f2d86cce5a1",
  "controls.json": "de63dc5c776f6286350015b64524e17ef9d91af77446e3e6f46af21884e36b01",
};

/** Authenticate real parent edges; shallow boundaries, grafts and replacements grant no ancestry. */
export function assertRawAncestor(root: string, required: string, cut: string) {
  for (const identity of [required, cut]) assert.match(identity, /^[0-9a-f]{40}$/u);
  const parents = (identity: string) => {
    const bytes = execFileSync("git", ["--no-replace-objects", "cat-file", "commit", identity], {
      cwd: root,
      env: { ...process.env, GIT_NO_REPLACE_OBJECTS: "1" },
      timeout: 30_000,
      maxBuffer: 4 * 1024 * 1024,
    });
    assert.equal(
      createHash("sha1").update(`commit ${bytes.length}\0`).update(bytes).digest("hex"),
      identity,
      "complete raw commit object identity",
    );
    const headers = bytes.toString("utf8").split("\n\n", 1)[0].split("\n");
    assert.equal(headers.filter((line) => /^tree [0-9a-f]{40}$/u.test(line)).length, 1);
    return headers
      .filter((line) => line.startsWith("parent "))
      .map((line) => {
        assert.match(line, /^parent [0-9a-f]{40}$/u);
        return line.slice(7);
      });
  };
  parents(required);
  const pending = [cut],
    seen = new Set<string>();
  for (let index = 0; index < pending.length; index++) {
    const identity = pending[index];
    if (seen.has(identity)) continue;
    assert.ok(seen.size < 10_000, "bounded complete raw ancestor search");
    seen.add(identity);
    const actualParents = parents(identity);
    if (identity === required) return;
    pending.push(...actualParents);
  }
  assert.fail("namespace production fix is absent from immutable source cut C");
}
export const boundIds = [
  "svg-a",
  "svg-filter",
  "svg-blur",
  "svg-view",
  "svg-switch",
  "foreign-html-a",
  "html-a",
];
export type PhysicalRow = Row & {
  boundIds?: string[];
  expectedNamespacePhases?: string[][];
  expectedClassPhases?: string[][];
  expectedColorPhases?: string[][];
};
export type Binding = {
  id: string;
  namespace: string;
  classNameType: string;
  classAttribute: string | null;
  classProperty: string;
  styleAttribute: string | null;
  styleColor: string;
  computedColor: string;
};
export type PhysicalTrace = {
  tree: unknown[];
  namespaces: string[];
  events: unknown[];
  bindings?: Binding[];
};

/** Campaign identities and denominators come from reviewed observer code, never caller counts. */
export function campaignDefinition(name: unknown) {
  assert.ok(name === "reported" || name === "svg-namespace", "unknown public Vapor campaign");
  return name === "reported"
    ? {
        name,
        schema: "vize-public-reported-vapor-v1",
        original: 6,
        authored: 2,
        rows: 8,
        observations: 16,
      }
    : {
        name,
        schema: "vize-public-svg-namespace-v1",
        original: 0,
        authored: 1,
        rows: 1,
        observations: 2,
      };
}

/** Keep the one authored corpus distinct from all six unchanged reporter controls. */
export function materializeNamespaceFixtures(root: string, read: (file: string) => Buffer) {
  const prefix = `tests/_fixtures/differential/compiler/${namespaceDirectory}`;
  const authorities: { file: string; sha256: string }[] = [];
  const retain = (file: string) => {
    const source = `${prefix}${file}`;
    const bytes = read(source);
    const suffix = source.slice("tests/_fixtures/differential/compiler/".length);
    const target = path.join(root, "original", suffix);
    fs.mkdirSync(path.dirname(target), { recursive: true });
    fs.writeFileSync(target, bytes, { flag: "wx" });
    authorities.push({ file: source, sha256: digest(bytes) });
    return bytes;
  };
  const manifest = JSON.parse(retain(".manifest.json").toString("utf8"));
  assert.equal(manifest.schema, "vize.differential.runtime-pack");
  assert.equal(manifest.version, 1);
  assert.equal(manifest.product, "compiler");
  assert.equal(manifest.issue, "https://github.com/ubugeeei-prod/vize/issues/8624");
  assert.equal(manifest.cases.length, 1, "one separately authored namespace case");
  const declared = manifest.cases[0];
  assert.equal(declared.id, `compiler/${namespaceDirectory}`);
  assert.equal(declared.inputs.root, namespaceDirectory);
  assert.deepEqual(
    declared.inputs.files.map((entry: { path: string }) => entry.path),
    ["App.vue.txt", "Child.vue.txt", "controls.json"],
  );
  const contents: Record<string, string> = {};
  for (const entry of declared.inputs.files) {
    const bytes = retain(`/${entry.path}`);
    assert.equal(digest(bytes), entry.sha256, `whole immutable ${entry.path}`);
    assert.equal(digest(bytes), namespaceHashes[entry.path], `reviewed original ${entry.path}`);
    contents[entry.path] = bytes.toString("utf8");
  }
  assert.equal(declared.reference.path, `${namespaceDirectory}/controls.json`);
  assert.equal(declared.reference.sha256, digest(Buffer.from(contents["controls.json"])));
  const controls = JSON.parse(contents["controls.json"]);
  assert.equal(controls.length, 1, "namespace campaign cannot replace reporter denominator");
  const row: PhysicalRow = {
    ...controls[0],
    issue: 8624,
    directory: namespaceDirectory,
    app: contents["App.vue.txt"],
    child: contents["Child.vue.txt"],
  };
  assert.equal(row.name, "inherited-svg-and-html-integration-bindings");
  assert.deepEqual(row.boundIds, boundIds, "five SVG and two HTML bindings");
  assert.equal(row.steps.length, 2);
  assert.ok(row.steps.every((step) => step.patch && Object.keys(step).length === 1));
  assert.equal(row.expectedTrees?.length, 4);
  assert.equal(row.expectedNamespacePhases?.length, 4);
  assert.equal(row.expectedClassPhases?.length, 4);
  assert.equal(
    row.expectedColorPhases?.length,
    2,
    "computed color excludes inherited clear defaults",
  );
  assert.deepEqual(
    row.expectedClassPhases!.slice(0, 3),
    ["node array active", "node selected", "node"].map((value) => Array(7).fill(value)),
  );
  assert.deepEqual(
    row.expectedColorPhases,
    ["rgb(200, 0, 0)", "rgb(0, 0, 200)"].map((value) => Array(7).fill(value)),
  );
  const namespaces = [
    "http://www.w3.org/1999/xhtml",
    ...Array(9).fill("http://www.w3.org/2000/svg"),
    "http://www.w3.org/1999/xhtml",
    "http://www.w3.org/1999/xhtml",
  ];
  assert.deepEqual(row.expectedNamespacePhases!.slice(0, 3), [namespaces, namespaces, namespaces]);
  assert.ok(row.expectedTrees!.slice(0, 3).every((tree) => Array.isArray(tree) && tree.length > 0));
  assert.deepEqual(row.expectedClassPhases!.at(-1), []);
  assert.deepEqual(row.expectedNamespacePhases!.at(-1), []);
  assert.deepEqual(row.expectedTrees!.at(-1), []);
  return { authorities, rows: [row] };
}

export function assertNamespaceTrace(row: PhysicalRow, trace: PhysicalTrace[]) {
  assert.equal(row.directory, namespaceDirectory);
  assert.deepEqual(row.boundIds, boundIds);
  assert.equal(trace.length, 4, "complete mount/update/clear/unmount phases");
  assert.deepEqual(
    trace.map((phase) => phase.tree),
    row.expectedTrees,
  );
  assert.deepEqual(
    trace.map((phase) => phase.namespaces),
    row.expectedNamespacePhases,
  );
  assert.ok(trace.every((phase) => phase.events.length === 0));
  for (const [phaseIndex, phase] of trace.entries()) {
    const bindings = phase.bindings;
    assert.ok(bindings, "physical binding observations required");
    assert.deepEqual(
      bindings.map((entry) => entry.id),
      phaseIndex === 3 ? [] : boundIds,
    );
    assert.deepEqual(
      bindings.map((entry) => entry.classAttribute),
      row.expectedClassPhases![phaseIndex],
    );
    for (const [index, entry] of bindings.entries()) {
      const svg = index < 5;
      assert.equal(entry.namespace, `http://www.w3.org/${svg ? "2000/svg" : "1999/xhtml"}`);
      assert.equal(entry.classNameType, svg ? "[object SVGAnimatedString]" : "[object String]");
      assert.equal(
        entry.classProperty,
        entry.classAttribute,
        "physical class property equals DOM attribute",
      );
      if (phaseIndex < 2) {
        assert.equal(entry.computedColor, row.expectedColorPhases![phaseIndex][index]);
        assert.equal(entry.styleColor, row.expectedColorPhases![phaseIndex][index]);
        assert.ok(entry.styleAttribute);
      } else {
        assert.equal(entry.styleAttribute, null, "cleared style attribute must be absent");
        assert.equal(entry.styleColor, "", "cleared inline color must be empty");
      }
    }
  }
}
