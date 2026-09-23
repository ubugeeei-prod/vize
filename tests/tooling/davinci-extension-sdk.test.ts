import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

// P6-2: the extension SDK depends on no vize implementation crate, carries the
// contract WIT verbatim, and its JS/TS mirror matches the released surface.

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (...parts: string[]) => fs.readFileSync(path.join(root, ...parts), "utf8");

type Dependency = { name: string; kind: string | null };
type CargoPackage = { id: string; name: string; dependencies: Dependency[] };
type ResolveNode = { id: string; deps: { pkg: string; dep_kinds: { kind: string | null }[] }[] };
type Metadata = {
  packages: CargoPackage[];
  workspace_members: string[];
  resolve: { nodes: ResolveNode[] };
};

/** The only crates the SDK may depend on (normal and build edges). */
const SDK_DEPENDENCIES = ["dlmalloc", "wit-bindgen"];

function sdkDependencyViolations(sdk: CargoPackage, workspaceCrates: Set<string>): string[] {
  return sdk.dependencies
    .filter((dependency) => dependency.kind !== "dev")
    .flatMap((dependency) => {
      if (workspaceCrates.has(dependency.name)) {
        return [`vize_extension_sdk depends on workspace crate ${dependency.name}`];
      }
      if (!SDK_DEPENDENCIES.includes(dependency.name)) {
        return [`vize_extension_sdk depends on unlisted crate ${dependency.name}`];
      }
      return [];
    });
}

const metadata = JSON.parse(
  execFileSync("cargo", ["metadata", "--format-version", "1"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
  }),
) as Metadata;
const members = new Set(metadata.workspace_members);
const workspaceCrates = new Set(
  metadata.packages.filter((pkg) => members.has(pkg.id)).map((pkg) => pkg.name),
);
const sdk = metadata.packages.find((pkg) => pkg.name === "vize_extension_sdk");

test("the SDK's direct dependencies are exactly the pinned guest set", () => {
  assert.ok(sdk, "vize_extension_sdk is a workspace member");
  assert.deepEqual(sdkDependencyViolations(sdk, workspaceCrates), []);
  assert.deepEqual(
    sdk.dependencies
      .filter((dependency) => dependency.kind !== "dev")
      .map((dependency) => dependency.name)
      .toSorted(),
    SDK_DEPENDENCIES,
  );
});

test("the dependency check fails on an injected implementation edge", () => {
  assert.ok(sdk);
  const injected = {
    ...sdk,
    dependencies: [
      ...sdk.dependencies,
      { name: "vize_s1", kind: null },
      { name: "serde", kind: "build" },
    ],
  };
  assert.deepEqual(sdkDependencyViolations(injected, workspaceCrates), [
    "vize_extension_sdk depends on workspace crate vize_s1",
    "vize_extension_sdk depends on unlisted crate serde",
  ]);
});

test("no workspace crate is reachable from the SDK", () => {
  assert.ok(sdk);
  const nodes = new Map(metadata.resolve.nodes.map((node) => [node.id, node]));
  const reached = new Set<string>();
  const queue = [sdk.id];
  while (queue.length > 0) {
    const node = nodes.get(queue.pop()!);
    for (const edge of node?.deps ?? []) {
      const shipped = edge.dep_kinds.some((kind) => kind.kind !== "dev");
      if (!shipped || reached.has(edge.pkg)) continue;
      reached.add(edge.pkg);
      queue.push(edge.pkg);
    }
  }
  assert.deepEqual(
    [...reached].filter((id) => members.has(id)),
    [],
  );
});

test("the SDK carries contracts/wit verbatim", () => {
  const files = (dir: string) => fs.readdirSync(path.join(root, dir)).toSorted();
  assert.deepEqual(files("crates/vize_extension_sdk/wit"), files("contracts/wit"));
  for (const file of files("contracts/wit")) {
    assert.equal(read("crates/vize_extension_sdk/wit", file), read("contracts/wit", file), file);
  }
});

type Surface = {
  version: string;
  protocolVersion: number;
  pages: Record<string, number>;
  interfaces: Record<
    string,
    {
      types: Record<string, Record<string, unknown>>;
      functions: Record<string, { params: { name: string; type: string }[]; result?: string }>;
    }
  >;
  worlds: Record<string, { requiredFeatures: string[] }>;
};

function newestSurface(): Surface {
  const files = fs.readdirSync(path.join(root, "contracts/versions")).toSorted();
  assert.deepEqual(files, [
    "vize-contracts@0.1.0.json",
    "vize-contracts@0.1.1.json",
    "vize-contracts@0.1.2.json",
  ]);
  return JSON.parse(read("contracts/versions", files.at(-1)!)) as Surface;
}

const pascal = (name: string) =>
  name.replace(/(?:^|-)([a-z])/gu, (_, letter: string) => letter.toUpperCase());
const camel = (name: string) =>
  name.replace(/-([a-z])/gu, (_, letter: string) => letter.toUpperCase());

function tsType(wit: string): string {
  const generic = /^(?<outer>list|option)<(?<inner>.+)>$/u.exec(wit);
  if (generic?.groups?.outer === "list") return `Array<${tsType(generic.groups.inner)}>`;
  if (generic?.groups?.outer === "option") return `${tsType(generic.groups.inner)} | undefined`;
  if (["u8", "u16", "u32", "s8", "s16", "s32", "f32", "f64"].includes(wit)) return "number";
  if (wit === "string" || wit === "bool") return wit === "bool" ? "boolean" : "string";
  return pascal(wit.split(".").at(-1)!);
}

/** The body lines of `export interface <name> {` in the declarations. */
function interfaceBody(declarations: string, name: string): string[] {
  const match = new RegExp(`^export interface ${name} \\{\\n(?<body>[\\s\\S]*?)^\\}`, "mu").exec(
    declarations,
  );
  assert.ok(match?.groups, `index.d.ts declares interface ${name}`);
  return match.groups.body
    .split("\n")
    .filter((line) => line.trim() !== "")
    .map((line) => line.trim());
}

test("the TypeScript declarations mirror the released surface", () => {
  const declarations = read("npm/extension-sdk/index.d.ts");
  const surface = newestSurface();
  let checked = 0;
  for (const iface of Object.values(surface.interfaces)) {
    for (const [name, shape] of Object.entries(iface.types)) {
      checked += 1;
      if ("record" in shape) {
        const fields = shape.record as { name: string; type: string }[];
        assert.deepEqual(
          interfaceBody(declarations, pascal(name)),
          fields.map((field) => `${camel(field.name)}: ${tsType(field.type)};`),
        );
      } else if ("enum" in shape) {
        const cases = (shape.enum as string[]).map((value) => `"${value}"`).join(" | ");
        assert.ok(declarations.includes(`export type ${pascal(name)} = ${cases};\n`), name);
      } else if ("variant" in shape) {
        const cases = shape.variant as { name: string; type?: string }[];
        const union = cases.map((item) => `${pascal(name)}${pascal(item.name)}`).join(" | ");
        assert.ok(declarations.includes(`export type ${pascal(name)} = ${union};\n`), name);
        for (const item of cases) {
          assert.deepEqual(interfaceBody(declarations, `${pascal(name)}${pascal(item.name)}`), [
            `tag: "${item.name}";`,
            ...(item.type ? [`val: ${tsType(item.type)};`] : []),
          ]);
        }
      } else {
        assert.fail(`${name}: an unmapped shape ${JSON.stringify(shape)}`);
      }
    }
  }
  for (const exported of ["handshake", "input-lowering", "expression-analysis", "emission"]) {
    const methods = Object.entries(surface.interfaces[exported].functions).map(
      ([name, fn]) =>
        `${camel(name)}(${fn.params.map((param) => `${camel(param.name)}: ${tsType(param.type)}`).join(", ")}): ${fn.result ? tsType(fn.result) : "void"};`,
    );
    assert.deepEqual(interfaceBody(declarations, pascal(exported)), methods);
  }
  assert.equal(checked, 17);
});

test("the JS and Rust SDK constants are the released handshake", async () => {
  const surface = newestSurface();
  const sdkJs = (await import(path.join(root, "npm/extension-sdk/index.js"))) as Record<
    string,
    unknown
  >;
  const rust = read("crates/vize_extension_sdk/src/lib.rs");
  const rustConst = (name: string) =>
    new RegExp(`^pub const ${name}: [^=]+ = (?<value>[^;]+);$`, "mu").exec(rust)?.groups?.value;
  assert.equal(sdkJs.PACKAGE, `vize:contracts@${surface.version}`);
  assert.equal(rustConst("PACKAGE"), `"vize:contracts@${surface.version}"`);
  assert.equal(sdkJs.PROTOCOL_VERSION, surface.protocolVersion);
  assert.equal(rustConst("PROTOCOL_VERSION"), `${surface.protocolVersion}`);
  assert.equal(sdkJs.S1_PAGE_SCHEMA, surface.pages["s1-page"]);
  assert.equal(rustConst("S1_PAGE_SCHEMA"), `${surface.pages["s1-page"]}`);
  assert.equal(sdkJs.S2_PAGE_SCHEMA, surface.pages["s2-page"]);
  assert.equal(rustConst("S2_PAGE_SCHEMA"), `${surface.pages["s2-page"]}`);
  const required = surface.worlds["input-dialect"].requiredFeatures;
  assert.deepEqual(sdkJs.REQUIRED_FEATURES, required);
  assert.equal(rustConst("REQUIRED_FEATURES"), `&[${required.map((f) => `"${f}"`).join(", ")}]`);
  for (const [name, page] of [
    ["FACTS_PAGE_SCHEMA", "facts-page"],
    ["PROJECTION_PAGE_SCHEMA", "projection-page"],
    ["S3_PAGE_SCHEMA", "s3-page"],
    ["EMIT_DOCUMENT_PAGE_SCHEMA", "emit-document-page"],
  ]) {
    assert.equal(sdkJs[name], surface.pages[page], name);
    assert.equal(rustConst(name), `${surface.pages[page]}`, name);
  }
  const outputRequired = surface.worlds["output-target"].requiredFeatures;
  assert.deepEqual(sdkJs.OUTPUT_REQUIRED_FEATURES, outputRequired);
  assert.equal(
    rustConst("OUTPUT_REQUIRED_FEATURES"),
    `&[${outputRequired.map((f) => `"${f}"`).join(", ")}]`,
  );
  const expressionRequired = surface.worlds["expression-dialect"].requiredFeatures;
  assert.deepEqual(sdkJs.EXPRESSION_REQUIRED_FEATURES, expressionRequired);
  assert.equal(
    rustConst("EXPRESSION_REQUIRED_FEATURES"),
    `&[${expressionRequired.map((f) => `"${f}"`).join(", ")}]`,
  );
  const capability = sdkJs.capability as (langs: string[]) => unknown;
  assert.deepEqual(capability(["zz", "html", "html"]), {
    protocolVersion: 1,
    features: ["lang:html", "lang:zz", "s1-page@1", "s2-page@1"],
  });
});
