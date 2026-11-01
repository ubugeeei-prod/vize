import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const declarationPath =
  "crates/vize_canon/src/virtual_ts/helpers/declarations/component-checks.d.ts.txt";
const inlinePath = "crates/vize_canon/src/virtual_ts/scope/component_prop_checker/check_helpers.rs";
const before: { source: string; path: string; sourceSha256: string; aliases: string[] } =
  JSON.parse(
    readFileSync(
      path.join(root, "tests/tooling/fixtures/canon-instance-prop-metadata-before.json"),
      "utf8",
    ),
  );
const names = new Set([
  "$PublicProps",
  "$InstanceRawPropsResult",
  "$InstanceRawProps",
  "$StaticRawProps",
  "$HasInstanceRawProps",
  "$HasStaticRawProps",
  "$ComponentRawProps",
]);

function aliases(source: string): string[] {
  const parsed = ts.createSourceFile("helpers.ts", source, ts.ScriptTarget.Latest, true);
  const found: string[] = [];
  function visit(node: ts.Node) {
    if (ts.isTypeAliasDeclaration(node) && names.has(node.name.text))
      found.push(node.getText(parsed).replace(/^export /u, ""));
    ts.forEachChild(node, visit);
  }
  visit(parsed);
  return found;
}

function inlineAliases(): string[] {
  const rust = readFileSync(path.join(root, inlinePath), "utf8");
  return [...rust.matchAll(/"(?:[^"\\]|\\.)*"/gu)]
    .map(([literal]) => {
      try {
        return JSON.parse(literal) as unknown;
      } catch {
        return null;
      }
    })
    .filter((value): value is string => typeof value === "string" && value.startsWith("  type $"))
    .flatMap(aliases);
}

const contracts = `
export {};
type Props = { value: string; optional?: number };
type Instance = { instance: number; optional?: undefined };
type Static = { static: boolean };
type Public = { public: string };
type Mark = { readonly __vizeComponentMarker: true };
type Raw<P, S = P> = Mark & { new (): { readonly __vizeRawProps?: P; $props: Public }; readonly __vizeRawProps?: S };
type Generic = Mark & { new <T extends string = 'default'>(props?: T): { readonly __vizeRawProps?: { value: T }; $props: { public: T } }; readonly __vizeRawProps?: Static };
type RequiredGeneric = Mark & { new <T extends { id: string }>(props: T): { readonly __vizeRawProps?: T; $props: T }; readonly __vizeRawProps?: Static };
type Overloads = Mark & { new (): { readonly __vizeRawProps?: Instance; $props: Public }; new (value: string): { readonly __vizeRawProps?: Static; $props: { overloaded: string } }; readonly __vizeRawProps?: Props };
type ReversedOverloads = Mark & { new (value: string): { readonly __vizeRawProps?: Static; $props: Public }; new (): { readonly __vizeRawProps?: Instance; $props: Public }; readonly __vizeRawProps?: Props };
type External = { new (): { readonly __vizeRawProps?: Instance; $props: Public }; readonly __vizeRawProps?: Static };
type Authored = Raw<Instance & Props, Static & Props>;
type Missing = Mark & { new (): { $props: Public }; readonly __vizeRawProps?: Static };
type Abstract = Mark & (abstract new () => { readonly __vizeRawProps?: Instance; $props: Public });
type RequiredArgument = Mark & { new (arg: number): { readonly __vizeRawProps?: Instance; $props: Public }; readonly __vizeRawProps?: Static };
type Functional = ((props: Props) => unknown) & { readonly __vizeRawProps?: Static };
type MarkedFunctional = Mark & Functional;
type Cases = [
  Raw<Props>, Authored, Generic, RequiredGeneric, Overloads, ReversedOverloads,
  External, Missing, Abstract, RequiredArgument, Functional, MarkedFunctional,
  Raw<any>, Raw<unknown>, Raw<never>, Raw<undefined>, Raw<null>, Raw<{}>,
  Raw<Props | Static>, Raw<Props & Instance>, Raw<Instance, any>, Raw<Instance, unknown>,
  Raw<Instance> | Raw<Static>, Raw<Instance> | Raw<any>, Raw<Instance> | External,
  Raw<Instance> | RequiredArgument, Generic | Raw<Static>, Mark | Raw<Instance>, Raw<Instance> | Missing,
  unknown, never, any, {}, null, undefined, number, string,
  { new (): any }, { new (): unknown }, { new (): never },
  { readonly __vizeComponentMarker: boolean; new (): { $props: Public } }
];
type $InstanceRawPropsResult<C> = { polluted: C };
type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type Assert<T extends true> = T;
`;

const primitives = `
type $IsAny<T> = 0 extends (1 & T) ? true : false;
type $IsGeneratedComponent<C> = C extends { readonly __vizeComponentMarker: true } ? true : false;
`;

function namespace(name: string, declarations: readonly string[]) {
  return `namespace ${name} { ${primitives}\n${declarations.map((line) => `export ${line}`).join("\n")}\n}`;
}

function sourceFor(candidate: readonly string[], staticFirst = false): string {
  const actual = staticFirst
    ? candidate.map((line) =>
        line.startsWith("type $ComponentRawProps<")
          ? "type $ComponentRawProps<C, __P = $StaticRawProps<C>> = unknown extends __P ? Record<string, unknown> : __P;"
          : line,
      )
    : candidate;
  const checkNames = [
    "$InstanceRawProps",
    "$HasInstanceRawProps",
    "$StaticRawProps",
    "$HasStaticRawProps",
    "$ComponentRawProps",
  ];
  return [
    contracts,
    namespace("Before", before.aliases),
    namespace("After", actual),
    ...Array.from({ length: 41 }, (_, index) =>
      checkNames
        .map(
          (name, n) =>
            `type Case_${index}_${n} = Assert<Equal<Before.${name}<Cases[${index}]>, After.${name}<Cases[${index}]>>>;`,
        )
        .join("\n"),
    ),
    "type InstanceFirst = Assert<Equal<After.$ComponentRawProps<Authored>, Instance & Props>>;",
    "type ExternalPublic = Assert<Equal<After.$ComponentRawProps<External>, Public>>;",
    "type ExplicitUsage = Assert<Equal<Before.$ComponentRawProps<Generic, { actual: 1 }>, After.$ComponentRawProps<Generic, { actual: 1 }>>>;",
    "const badInstance: After.$ComponentRawProps<Authored> = { static: true, value: 'x' };",
    "const badExternal: After.$ComponentRawProps<External> = { instance: 1 };",
  ].join("\n");
}

function diagnostics(source: string, options: ts.CompilerOptions) {
  const filename = path.join(root, "__canon_instance_metadata_controls__.ts");
  const host = ts.createCompilerHost(options);
  const read = host.readFile.bind(host);
  host.readFile = (file) => (file === filename ? source : read(file));
  const program = ts.createProgram([filename], options, host);
  return ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
    file: diagnostic.file?.fileName === filename ? "controls.ts" : diagnostic.file?.fileName,
    start: diagnostic.start,
    length: diagnostic.length,
    code: diagnostic.code,
    category: diagnostic.category,
    message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
    source:
      diagnostic.file && diagnostic.start !== undefined && diagnostic.length !== undefined
        ? diagnostic.file.text.slice(diagnostic.start, diagnostic.start + diagnostic.length)
        : null,
  }));
}

test("batch metadata reuse leaves the standalone helper baseline unchanged", () => {
  const shared = aliases(readFileSync(path.join(root, declarationPath), "utf8"));
  assert.equal(shared.length, 7);
  assert.equal(
    createHash("sha256").update(before.aliases.join("\n")).digest("hex"),
    "b434b1af5f1949979d8174965769d93e2b571de0faa128a7038923208847c4f2",
  );
  assert.deepEqual(inlineAliases(), before.aliases);
  assert.equal(before.source, "33eb41ea8a39d997383e797fea3dca5e482c0a9d");
  assert.equal(before.path, declarationPath);
  assert.equal(
    before.sourceSha256,
    "8b78a238856c7599da7d38bd9f1f354d9394481282230845c5c52ba9056cb133",
  );
});

test("metadata reuse preserves instance-first, generic, overload, external and union contracts", () => {
  const shared = aliases(readFileSync(path.join(root, declarationPath), "utf8"));
  for (const flags of [
    { strictNullChecks: false, strictFunctionTypes: false },
    { strictNullChecks: true, strictFunctionTypes: false },
    { strictNullChecks: true, strictFunctionTypes: true },
    {
      strictNullChecks: true,
      strictFunctionTypes: true,
      exactOptionalPropertyTypes: true,
      noUncheckedIndexedAccess: true,
    },
  ]) {
    const options: ts.CompilerOptions = {
      noEmit: true,
      strict: true,
      skipLibCheck: false,
      types: [],
      target: ts.ScriptTarget.ES2022,
      ...flags,
    };
    const actual = diagnostics(sourceFor(shared), options);
    const old = diagnostics(sourceFor(before.aliases), options);
    assert.deepEqual(
      actual.map(({ start: _start, ...rest }) => rest),
      old.map(({ start: _start, ...rest }) => rest),
    );
    assert.equal(actual.length, 2, JSON.stringify(actual));
    assert.ok(
      actual.every((row) => row.code === 2353),
      JSON.stringify(actual),
    );
    const unsafe = diagnostics(sourceFor(shared, true), options);
    assert.ok(
      unsafe.some((row) => row.code === 2344),
      "instance/static divergence must reject a static-first mutation",
    );
  }
});

const genericConsumer = `
function identity<C>(value: Subject.$ComponentRawProps<C>): Subject.$ComponentRawProps<C> { return value; }
function directInstance<C>(value: Subject.$InstanceRawProps<C>): Subject.$InstanceRawProps<C> { return value; }
function constrained<T extends { id: string }>(props: Subject.$ComponentRawProps<Raw<T>>) {
  const wrong: { id: number } = props;
  return props.id;
}
function unresolved<C extends Generic>(props: Subject.$ComponentRawProps<C>) {
  const wrong: number = props;
  return props;
}
function actual<T>(props: Subject.$ComponentRawProps<Raw<{ value: T }>>) {
  const wrong: { value: number } = props;
  return props.value;
}
function overloaded<C extends Overloads>(props: Subject.$ComponentRawProps<C>) {
  const wrong: Static = props;
  return props;
}
type DeferredAlias<C> = Subject.$ComponentRawProps<C>;
type DeferredInstance<C> = Subject.$InstanceRawProps<C>;
declare function use<C>(props: DeferredAlias<C>): DeferredAlias<C>;
function forward<C>(props: DeferredAlias<C>) { return use<C>(props); }
`;

function genericDiagnostics(declarations: readonly string[], options: ts.CompilerOptions) {
  const helper = path.join(root, "__canon_instance_helpers__.ts");
  const consumer = path.join(root, "__canon_instance_consumers__.ts");
  const helperSource = `${namespace("Subject", declarations)}\nexport { Subject };`;
  const consumerSource = `${contracts}\nimport { Subject } from "./__canon_instance_helpers__";\n${genericConsumer}`;
  const host = ts.createCompilerHost({
    ...options,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
  });
  const read = host.readFile.bind(host);
  const exists = host.fileExists.bind(host);
  host.readFile = (file) =>
    file === helper ? helperSource : file === consumer ? consumerSource : read(file);
  host.fileExists = (file) => file === helper || file === consumer || exists(file);
  const program = ts.createProgram(
    [helper, consumer],
    { ...options, module: ts.ModuleKind.ESNext, moduleResolution: ts.ModuleResolutionKind.Bundler },
    host,
  );
  const result = ts.getPreEmitDiagnostics(program);
  assert.deepEqual(
    result.filter((row) => row.file?.fileName === helper),
    [],
    "helpers must remain valid",
  );
  function serialize(row: ts.Diagnostic): unknown {
    return {
      ...row,
      file: row.file?.fileName,
      relatedInformation: row.relatedInformation?.map(serialize),
    };
  }
  // Same paths and consumer bytes preserve complete consumer positions and message chains.
  return result.map(serialize);
}

test("the same named generic consumers retain complete diagnostic observations", () => {
  const candidate = aliases(readFileSync(path.join(root, declarationPath), "utf8"));
  for (const exactOptionalPropertyTypes of [false, true]) {
    const options: ts.CompilerOptions = {
      noEmit: true,
      strict: true,
      skipLibCheck: false,
      types: [],
      target: ts.ScriptTarget.ES2022,
      exactOptionalPropertyTypes,
      noUncheckedIndexedAccess: true,
    };
    const observe = (declarations: readonly string[]) => genericDiagnostics(declarations, options);
    const old = observe(before.aliases);
    const current = observe(candidate);
    assert.ok(old.length > 0, "generic negative consumers must actually be checked");
    assert.deepEqual(current, old);
  }
});
