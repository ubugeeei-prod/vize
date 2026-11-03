import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

// #11699: a component constructor replaced `$props` across the whole instance,
// so each use remapped every `ComponentCustomProperties` member. It now
// replaces them on the component's own members and intersects the base as is.
const repo = fileURLToPath(new URL("../../", import.meta.url));
const directory = fileURLToPath(new URL("./__instance_split__/", import.meta.url));
const helperLine = (relative: string, name: string): string => {
  const found = fs
    .readFileSync(`${repo}crates/vize_canon/src/virtual_ts/${relative}`, "utf8")
    .split("\n")
    .find((text) => text.startsWith(`type ${name}<`));
  assert.ok(found, name);
  return found;
};
const helpers = [
  helperLine("component_prop_helpers.txt", "$LiteralKeys"),
  helperLine("component_prop_helpers.txt", "$ReplaceComponentProps"),
  helperLine("component_prop_helpers.txt", "$ComponentPublicBase"),
  helperLine("helpers/declarations/vue3-types.d.ts.txt", "$MappedOmit"),
  helperLine("helpers/declarations/exposed-unwrap.d.ts.txt", "$ShallowUnwrapRef"),
].join("\n");

function check(files: Record<string, string>): { diagnostics: string[]; instantiations: number } {
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    lib: ["lib.esnext.d.ts", "lib.dom.d.ts"],
    types: [],
  };
  const virtual = new Map(
    Object.entries({ "helpers.d.ts": helpers, ...files }).map(([name, text]) => [
      `${directory}${name}`,
      text,
    ]),
  );
  const host = ts.createCompilerHost(options);
  const [read, exists] = [host.readFile.bind(host), host.fileExists.bind(host)];
  const directories = new Set([...virtual.keys()].map(dirname));
  const directoryExists = host.directoryExists!.bind(host);
  host.readFile = (file) => virtual.get(file) ?? read(file);
  host.fileExists = (file) => virtual.has(file) || exists(file);
  host.directoryExists = (directory) => directories.has(directory) || directoryExists(directory);
  const program = ts.createProgram([...virtual.keys()], options, host);
  const diagnostics = ts.getPreEmitDiagnostics(program).map((diagnostic) => {
    const name = diagnostic.file?.fileName.slice(directory.length) ?? "?";
    const where = diagnostic.file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0);
    const text = ts.flattenDiagnosticMessageText(diagnostic.messageText, " ");
    return `${name}:${where ? where.line + 1 : 0} TS${diagnostic.code} ${text}`;
  });
  return { diagnostics, instantiations: program.getInstantiationCount() };
}

const augment = (members: string) =>
  `export {};\ndeclare module "vue" {\n  interface ComponentCustomProperties {\n${members}\n  }\n}\n`;

type Form = "old" | "new" | "new with a base alias" | "renamed base" | "picked own" | "inline own";
const bases: Record<Form, string> = {
  old: "$ComponentPublicBase",
  new: "$ComponentPublicBase",
  "new with a base alias":
    "$MappedOmit<$ComponentPublicBase, '$el' | '$refs'> & { $el: HTMLDivElement; $refs: {} } & $ShallowUnwrapRef<{ open: () => void }>",
  "renamed base": "$ComponentPublicBase",
  "picked own": "$ComponentPublicBase",
  "inline own": "$ComponentPublicBase",
};

// One generated SFC's instance and constructor result, as the generator emits them.
function component(index: number, form: Form): string {
  const own = `{ $props: { a: number; k${index}: string }; readonly __vizeRawProps?: { a: number }; $emit: (event: "x") => void; $slots: {} }`;
  const base = ["new", "picked own", "inline own"].includes(form)
    ? bases.new
    : "$ComponentInstanceBase";
  const declarations =
    form === "old"
      ? [
          `type $ComponentInstance = ${own} & ${bases.old};`,
          "declare const i: $ReplaceComponentProps<$ComponentInstance, { b: string }>;",
        ]
      : form === "inline own"
        ? [
            `type $ComponentInstance = ${own} & ${bases.new};`,
            `declare const i: $ReplaceComponentProps<${own}, { b: string }> & ${base};`,
          ]
        : form === "picked own"
          ? [
              `type $ComponentInstance = ${own} & ${bases.new};`,
              "type $ComponentOwnInstance = globalThis.Pick<$ComponentInstance, '$props' | '__vizeRawProps' | '$emit' | '$slots'>;",
              `declare const i: $ReplaceComponentProps<$ComponentOwnInstance, { b: string }> & ${base};`,
            ]
          : [
              `type $ComponentOwnInstance = ${own};`,
              ...(form === "new" ? [] : [`type $ComponentInstanceBase = ${bases[form]};`]),
              `type $ComponentInstance = $ComponentOwnInstance & ${base};`,
              `declare const i: $ReplaceComponentProps<$ComponentOwnInstance, { b: string }> & ${base};`,
            ];
  return [
    ...declarations,
    "declare const j: $ComponentInstance;",
    "export const v = [i.$attrs, i.$props.b, i.$emit, i.$el, i.m0, j.$props.a, j.m1];",
  ].join("\n");
}

function scale(files: number, form: Form, members: number): number {
  const sources: Record<string, string> = {
    "members.ts": augment(
      Array.from({ length: members }, (_, index) => `    readonly m${index}: number;`).join("\n"),
    ),
  };
  for (let index = 0; index < files; index += 1) {
    sources[`c${index}.ts`] = component(index, form);
  }
  const { diagnostics, instantiations } = check(sources);
  assert.deepEqual(diagnostics, [], `${form} ${files}`);
  return instantiations;
}

void test("each component use costs instantiations independent of the custom property count", (t) => {
  const members = 2000;
  const marginal = (form: Form): [number, string] => {
    const counts = [scale(100, form, members), scale(400, form, members)];
    return [(counts[1]! - counts[0]!) / 300, `${form}: ${counts.join(" -> ")} instantiations`];
  };
  // 100 -> 400 SFCs: about 0.08M -> 0.12M against 2.1M -> 8.2M for the old form.
  for (const form of ["new", "new with a base alias", "picked own", "inline own"] as const) {
    const [perComponent, detail] = marginal(form);
    t.diagnostic(detail);
    assert.ok(perComponent < members / 10, detail);
  }
  // Negative controls. The full-instance remap pays for every member per
  // component. So does an alias that only renames `$ComponentPublicBase`:
  // TypeScript instantiates the helper again under each new alias name.
  for (const form of ["old", "renamed base"] as const) {
    const [perComponent, detail] = marginal(form);
    t.diagnostic(detail);
    assert.ok(perComponent > members, detail);
  }
});

const prelude = [
  'import type { Ref, ComponentPublicInstance } from "vue";',
  "type Equal<A, B> = (<G>() => G extends A ? 1 : 2) extends <G>() => G extends B ? 1 : 2 ? true : false;",
  "type Expect<T extends true> = T;",
  // Key remapping keeps literal members beside index signatures and modifiers.
  "type Flat<T> = { [K in keyof T as K]: T[K] };",
  "type At<T, K> = K extends keyof T ? T[K] : 0;",
  "type Props = { b: string } & { a?: number };",
  "type OwnOf<P, E, S> = { $props: P; readonly __vizeRawProps?: P; $emit: E; $slots: S };",
  'type Own = OwnOf<{ a: number }, (event: "x", value: number) => void, { default?: () => unknown }>;',
  'type IndexedOwn = { [key: string]: unknown; $props: { a: number }; readonly __vizeRawProps?: { a: number }; $emit: (event: "x") => void; $slots: {} };',
  'type GenericOwn<T> = OwnOf<{ value: T; items: T[] }, (event: "pick", value: T) => void, { item?: (props: { item: T }) => unknown }>;',
  "type Exposed = { open: () => void; count: Ref<number> };",
  "type Root = { $el: HTMLDivElement };",
  "type Refs = { $refs: { input: HTMLInputElement | null } };",
  "type Authored = ComponentPublicInstance<{ title: string }, {}, { count: number }, {}, { inc(): void }>;",
  "type AuthoredBase = Omit<Authored, '$props' | '$emit' | '$slots'>;",
];

// `[name, own, base, previous instance]`: the previous instance spells the
// authored default's `Omit<...>` first, as the generator did.
const shapes: [string, string, string, string?][] = [
  ["plain", "Own", "$ComponentPublicBase"],
  ["indexed own", "IndexedOwn", "$ComponentPublicBase"],
  // Indexed props stay inside `$props`; the own part keeps its four literal keys.
  ...[
    "Record<string, unknown>",
    "{ [key: string]: unknown; a?: number }",
    "{ [key: `${string}`]: number }",
  ].map((props): [string, string, string] => [
    `indexed props ${props}`,
    `OwnOf<${props}, (event: "x") => void, {}>`,
    "$ComponentPublicBase",
  ]),
  ...["string", "number[]", "unknown", "any", "never", "{ id: 1 } | undefined"].map(
    (argument): [string, string, string] => [
      `generic ${argument}`,
      `GenericOwn<${argument}>`,
      "$ComponentPublicBase",
    ],
  ),
  ["exposed", "Own", "$ComponentPublicBase & $ShallowUnwrapRef<Exposed>"],
  ["$el", "Own", "$MappedOmit<$ComponentPublicBase, '$el'> & Root"],
  ["$refs", "Own", "$MappedOmit<$ComponentPublicBase, '$refs'> & Refs"],
  ["$el and $refs", "Own", "$MappedOmit<$ComponentPublicBase, '$el' | '$refs'> & Root & Refs"],
  [
    "$el, $refs and exposed",
    "Own",
    "$MappedOmit<$ComponentPublicBase, '$el' | '$refs'> & Root & Refs & $ShallowUnwrapRef<Exposed>",
  ],
  [
    "authored default",
    "Own",
    "AuthoredBase & $ComponentPublicBase",
    "AuthoredBase & Own & $ComponentPublicBase",
  ],
  [
    "authored default with exposed",
    "Own",
    "AuthoredBase & $ComponentPublicBase & $ShallowUnwrapRef<Exposed>",
    "AuthoredBase & Own & $ComponentPublicBase & $ShallowUnwrapRef<Exposed>",
  ],
];

// Reads through every literal key and index signature, so a shape whose
// whole type cannot be compared still pins what each member access sees.
const keys = [
  ...["$props", "$emit", "$slots", "__vizeRawProps", "$attrs", "$el", "$refs", "$data"],
  ...["$options", "$watch", "$route", "useFeature", "open", "count", "inc", "title"],
].map((key) => `'${key}'`);
const indexKeys = ["string", "number", "`$${string}`", "`$x${string}`", "'$unknown'", "'unknown'"];

function equalities(pattern: boolean, pickOwn = false): string {
  return [
    ...prelude,
    ...shapes.flatMap(([name, own, base, previous = `${own} & ${base}`], index) => {
      // The generated own part has exactly four keys; an arbitrary indexed
      // own type is covered separately by the direct-intersection oracle.
      if (pickOwn && name === "indexed own") return [];
      const [oldInstance, newInstance] = [`Old${index}`, `New${index}`];
      // An index-signature own type makes the full-instance `Omit` collapse to
      // `string`, which dropped the base's `$${string}` signature; intersecting
      // the base unmapped keeps it. Generated own parts have no index signature.
      const whole = !(pattern && name === "indexed own");
      return [
        `type ${oldInstance} = ${previous};`,
        `type ${newInstance} = ${own} & (${base});`,
        `type OldUse${index} = $ReplaceComponentProps<${oldInstance}, Props>;`,
        `type NewUse${index} = $ReplaceComponentProps<${pickOwn ? `Pick<${oldInstance}, '$props' | '__vizeRawProps' | '$emit' | '$slots'>` : own}, Props> & (${base});`,
        `type _instance${index} = Expect<Equal<Flat<${newInstance}>, Flat<${oldInstance}>>>;`,
        ...(whole
          ? [`type _use${index} = Expect<Equal<Flat<NewUse${index}>, Flat<OldUse${index}>>>;`]
          : []),
        `type _keys${index} = Expect<Equal<keyof NewUse${index}, keyof OldUse${index}>>;`,
        ...[...keys, ...indexKeys].map(
          (key, at) =>
            `type _${index}_${at} = Expect<Equal<At<NewUse${index}, ${key}>, At<OldUse${index}, ${key}>>>;`,
        ),
        `declare const newUse${index}: NewUse${index};`,
        `declare const oldUse${index}: OldUse${index};`,
        `export const toOld${index}: OldUse${index} = newUse${index};`,
        `export const toNew${index}: NewUse${index} = oldUse${index};`,
      ];
    }),
  ].join("\n");
}

const customProperties: [string, string][] = [
  ["plain members", "    readonly $route: { path: string };\n    useFeature: () => number;"],
  ["a string index signature", "    [key: string]: any;\n    useFeature: () => number;"],
  [
    "a `$${string}` pattern (Nuxt)",
    "    [key: `$${string}`]: unknown;\n    $route: { path: string };\n    useFeature: () => number;",
  ],
];

void test("own-part replacement types every instance member as the full-instance replacement did", () => {
  for (const [name, members] of customProperties) {
    const { diagnostics } = check({
      "members.ts": augment(members),
      "oracle.ts": equalities(name.includes("pattern")),
    });
    assert.deepEqual(diagnostics, [], name);
  }
});

void test("selecting the generated instance's four owned members preserves every member", () => {
  for (const [name, members] of customProperties) {
    const { diagnostics } = check({
      "members.ts": augment(members),
      "oracle.ts": equalities(name.includes("pattern"), true),
    });
    assert.deepEqual(diagnostics, [], name);
  }
});

void test("the owned-member selection cannot capture an authored Pick type", () => {
  const members = augment("readonly m0: number; readonly m1: number;");
  const authored = `type Pick = never;\n${component(0, "inline own")}`;
  const { diagnostics } = check({
    "members.ts": members,
    "shadow.ts": authored,
  });
  assert.deepEqual(diagnostics, []);
  const wrongProps = check({
    "members.ts": members,
    "shadow.ts": `${authored}\nconst wrong: typeof i.$props = { b: 123 };`,
  });
  assert.equal(wrongProps.diagnostics.length, 1);
  assert.match(
    wrongProps.diagnostics[0]!,
    /TS2322 Type 'number' is not assignable to type 'string'/u,
  );
  const captured = check({
    "members.ts": members,
    "shadow.ts": `type Pick = never;\n${component(0, "picked own").replace("globalThis.Pick", "Pick")}`,
  });
  assert.equal(captured.diagnostics.length, 1);
  assert.match(captured.diagnostics[0]!, /TS2315 Type 'Pick' is not generic/u);
});

void test("owned members cannot capture an authored globalThis namespace", () => {
  const authored = `import type * as globalThis from './authored';\n${component(0, "inline own")}`;
  const files = {
    "members.ts": augment("readonly m0: number; readonly m1: number;"),
    "authored.ts": "export interface Authored { id: number }\n",
    "component.ts": authored,
  };
  assert.deepEqual(check(files).diagnostics, []);
  const wrong = check({
    ...files,
    "component.ts": `${authored}\nconst wrong: typeof i.$props = { b: 123 };`,
  });
  assert.equal(wrong.diagnostics.length, 1);
  assert.match(wrong.diagnostics[0]!, /TS2322 Type 'number' is not assignable to type 'string'/u);
  const captured = check({
    ...files,
    "component.ts": `import type * as globalThis from './authored';\n${component(0, "picked own")}`,
  });
  assert.equal(captured.diagnostics.length, 1);
  assert.match(captured.diagnostics[0]!, /TS2694 Namespace .* has no exported member 'Pick'/u);
});

void test("the oracle rejects a replacement that drops the public base", () => {
  const broken = equalities(false).replaceAll(/(type NewUse\d+ = .*?) & \(.*\);$/gmu, "$1;");
  const { diagnostics } = check({
    "members.ts": augment(customProperties[0]![1]),
    "oracle.ts": broken,
  });
  assert.ok(
    diagnostics.some((text) => / TS2344 /u.test(text)),
    diagnostics.join("\n"),
  );
});

void test("the current generator never remaps a whole component instance", () => {
  const tracked = execFileSync(
    "git",
    ["ls-files", "-z", "--", "crates/vize_canon/src/virtual_ts/generator"],
    { cwd: repo, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 },
  )
    .split("\0")
    .filter((file) => file.endsWith(".rs"));
  const stale = tracked.filter((file) =>
    fs
      .readFileSync(`${repo}${file}`, "utf8")
      .includes("$ReplaceComponentProps<$ComponentInstance,"),
  );
  assert.deepEqual(stale, []);
});
