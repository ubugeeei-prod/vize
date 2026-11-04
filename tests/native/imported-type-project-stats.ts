import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { fileURLToPath } from "node:url";

type Native = Pick<
  typeof import("../../npm/native/index.js"),
  | "compileSfc"
  | "compileSfcBatch"
  | "getNativeBuildToken"
  | "typeProjectInputs"
  | "typeProjectOwners"
>;
type CompileResult = ReturnType<Native["compileSfc"]>;
type Context = NonNullable<CompileResult["typeProject"]>;
const source =
  '<script setup lang="ts">import type { FooProps } from "@lib/FooProps"; const props = defineProps<FooProps>();</script><template><div>{{ props.bar }}</div></template>';

const inputWrites: { path: string; sha256: string; content: string }[] = [];
let flush: () => void = () => {};
const projects = new Map<string, Map<string, string>>();
const observations: {
  api: string;
  args: unknown[];
  inputPins: ReturnType<typeof validateInputs>;
  result?: unknown;
  error?: string;
}[] = [];

function observed(native: Native): Native {
  return new Proxy(native, {
    get(target, key: string) {
      const method: unknown = target[key as keyof Native];
      if (typeof method !== "function") return method;
      return (...args: unknown[]) => {
        const inputPins = validateInputs();
        try {
          const result: unknown = Reflect.apply(method, target, args);
          observations.push({ api: key, args, inputPins, result });
          flush();
          return result;
        } catch (error) {
          observations.push({ api: key, args, inputPins, error: String(error) });
          flush();
          throw error;
        }
      };
    },
  });
}

function write(root: string, file: string, content: string) {
  const destination = path.join(root, file);
  fs.mkdirSync(path.dirname(destination), { recursive: true });
  fs.writeFileSync(destination, content);
  const inventory = projects.get(root) ?? new Map<string, string>();
  inventory.set(file, content);
  projects.set(root, inventory);
  inputWrites.push({
    path: destination,
    sha256: createHash("sha256").update(content).digest("hex"),
    content,
  });
  flush();
}

function validateInputs() {
  return [...projects].map(([root, expected]) => {
    const actual = fs
      .readdirSync(root, { recursive: true, withFileTypes: true })
      .filter((entry) => !entry.isDirectory())
      .map((entry) =>
        path.relative(root, path.join(entry.parentPath, entry.name)).split(path.sep).join("/"),
      )
      .sort();
    assert.deepEqual(actual, [...expected.keys()].sort(), "unknown or missing project input");
    const files = [...expected].map(([file, content]) => {
      const bytes = fs.readFileSync(path.join(root, file));
      assert.equal(
        bytes.toString("utf8"),
        content,
        "materialized input differs from authored fixture",
      );
      return { path: file, sha256: createHash("sha256").update(bytes).digest("hex") };
    });
    return { root, files };
  });
}

function runtimeProps(result: CompileResult) {
  assert.deepEqual(result.errors, []);
  const component = new Function("Vue", result.code)({}) as {
    props: Record<string, { type: { name: string }; required: boolean }>;
  };
  return Object.fromEntries(
    Object.entries(component.props).map(([name, value]) => [
      name,
      { type: value.type.name, required: value.required },
    ]),
  );
}

function prepareCases(root: string, unresolvedFirst = false) {
  write(
    root,
    "tsconfig.json",
    JSON.stringify({ files: [], references: [{ path: "./first" }, { path: "./second" }] }),
  );
  const cases = [];
  for (const [project, name, type, constructor] of [
    ["first", "Card", "Record<string, unknown>", "Object"],
    ["second", "Pane", "boolean", "Boolean"],
  ]) {
    assert(project && name && type && constructor);
    write(
      root,
      `${project}/tsconfig.json`,
      JSON.stringify({
        include: [`../app/${name}.vue`],
        compilerOptions: {
          paths: {
            "@lib/*": [unresolvedFirst && project === "first" ? "./missing/*" : "./types/*"],
          },
        },
      }),
    );
    write(root, `${project}/types/FooProps.ts`, `export interface FooProps { bar?: ${type} }`);
    const filename = path.join(root, `app/${name}.vue`);
    write(root, `app/${name}.vue`, source);
    cases.push({ filename, constructor });
  }
  return cases;
}

export function captureImportedTypeProjectStats(native: Native, root: string) {
  const cases = prepareCases(root);
  const singles = cases.map(({ filename, constructor }) => {
    const result = native.compileSfc(source, { filename, mode: "function" });
    const props = runtimeProps(result);
    assert.deepEqual(props, { bar: { type: constructor, required: false } });
    return { filename, result, props, stats: native.compileSfcBatch(filename, { threads: 1 }) };
  });
  return {
    singles,
    combined: native.compileSfcBatch(path.join(root, "app/*.vue"), { threads: 1 }),
  };
}

export function captureUnresolvedImportedTypeStats(native: Native, root: string) {
  const cases = prepareCases(root, true);
  assert(cases[0]);
  const result = native.compileSfc(source, { filename: cases[0].filename });
  assert.deepEqual(result.errors, [
    "Cannot resolve imported macro type `FooProps` from `@lib/FooProps`.",
  ]);
  return {
    result,
    singles: cases.map(({ filename }) => native.compileSfcBatch(filename, { threads: 1 })),
    combined: native.compileSfcBatch(path.join(root, "app/*.vue"), { threads: 1 }),
  };
}

export function assertImportedTypeProjectStats(
  packet: ReturnType<typeof captureImportedTypeProjectStats> & {
    unresolved?: ReturnType<typeof captureUnresolvedImportedTypeStats>;
  },
) {
  for (const { stats } of packet.singles) assert.deepEqual([stats.success, stats.failed], [1, 0]);
  assert(packet.singles[0] && packet.singles[1]);
  assert.notEqual(packet.singles[0].stats.outputBytes, packet.singles[1].stats.outputBytes);
  assert.deepEqual([packet.combined.success, packet.combined.failed], [2, 0]);
  assert.equal(
    packet.combined.outputBytes,
    packet.singles.reduce((sum, row) => sum + row.stats.outputBytes, 0),
  );
  if (packet.unresolved) {
    assert.deepEqual(
      packet.unresolved.singles.map((row: ReturnType<Native["compileSfcBatch"]>) => [
        row.success,
        row.failed,
      ]),
      [
        [0, 1],
        [1, 0],
      ],
    );
    assert.deepEqual(
      [packet.unresolved.combined.success, packet.unresolved.combined.failed],
      [1, 1],
    );
    assert(packet.unresolved.singles[1]);
    assert.equal(packet.unresolved.combined.outputBytes, packet.unresolved.singles[1].outputBytes);
  }
}

function context(result: CompileResult): Context {
  assert(result.typeProject, "configured compilation must retain captured authority");
  return result.typeProject;
}

export function captureProjectUpdates(native: Native, root: string) {
  const [first] = prepareCases(root);
  assert(first);
  const compile = () => native.compileSfc(source, { filename: first.filename, mode: "function" });
  const before = compile();
  const initial = context(before);
  const inputs = native.typeProjectInputs(initial.root, initial.revision);
  assert(inputs?.includes(path.join(root, "first/tsconfig.json")));
  assert.deepEqual(native.typeProjectInputs(initial.root, initial.revision, initial.revision), []);
  assert.deepEqual(native.typeProjectOwners(path.join(root, "first/tsconfig.json")), [
    initial.root,
  ]);
  write(root, "first/next/FooProps.ts", "export interface FooProps { bar?: string }");
  write(
    root,
    "first/tsconfig.json",
    JSON.stringify({
      include: ["../app/Card.vue"],
      compilerOptions: { paths: { "@lib/*": ["./next/*"] } },
    }),
  );
  const configured = compile();
  assert.deepEqual(runtimeProps(configured), { bar: { type: "String", required: false } });
  assert.notEqual(context(configured).revision, initial.revision);
  assert.equal(native.typeProjectInputs(initial.root, initial.revision), null);
  write(root, "first/next/FooProps.ts", "export interface FooProps { bar?: number }");
  const typeEdit = compile();
  assert.deepEqual(runtimeProps(typeEdit), { bar: { type: "Number", required: false } });
  write(
    root,
    "first/tsconfig.json",
    JSON.stringify({ files: [], compilerOptions: { paths: { "@lib/*": ["./next/*"] } } }),
  );
  write(
    root,
    "second/tsconfig.json",
    JSON.stringify({
      include: ["../app/*.vue"],
      compilerOptions: { paths: { "@lib/*": ["./types/*"] } },
    }),
  );
  const ownershipEdit = compile();
  assert.deepEqual(runtimeProps(ownershipEdit), { bar: { type: "Boolean", required: false } });
  return { before, inputs, configured, typeEdit, ownershipEdit };
}

function sourceIdentity(binding: string, native: Native) {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
  const sourceSha = git("rev-parse", "HEAD");
  const treeSha = git("rev-parse", "HEAD^{tree}");
  const trackedChanges = git("diff", "--name-only", "HEAD").split("\n").filter(Boolean);
  const token = native.getNativeBuildToken();
  assert(token, "source native build token is required");
  assert.match(token, /^[0-9a-f]{32}$/u);
  const required = process.env.VIZE_IMPORTED_TYPES_REQUIRE_SOURCE === "1";
  if (required) {
    assert.equal(sourceSha, process.env.VIZE_IMPORTED_TYPES_SOURCE_SHA);
    assert.equal(sourceSha, process.env.GITHUB_SHA);
    assert(
      trackedChanges.every((name) =>
        ["npm/native/index.js", "npm/native/index.d.ts"].includes(name),
      ),
    );
    assert(path.isAbsolute(binding));
    const binaries = fs
      .readdirSync(path.join(root, "npm/native"))
      .filter((name) => name.endsWith(".node"));
    assert.equal(binaries.length, 1);
    assert.equal(
      fs.realpathSync(binding),
      fs.realpathSync(path.join(root, "npm/native", binaries[0]!)),
    );
  }
  return {
    sourceSha,
    treeSha,
    trackedChanges,
    sourcePhase: required ? "source-pr-or-mq" : "local-functional",
    event: process.env.GITHUB_EVENT_NAME ?? null,
    physicalImage: fs.realpathSync(binding),
    bindingSha256: createHash("sha256").update(fs.readFileSync(binding)).digest("hex"),
    token,
  };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [binding, output] = process.argv.slice(2);
  assert(binding && output, "native binding and output receipt paths are required");
  assert(path.isAbsolute(output), "receipt path must be absolute");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const receipt: Record<string, unknown> = {
    schema: "vize.compiler.imported-type-project",
    version: 1,
    status: "attempt",
    node: process.version,
  };
  receipt.inputWrites = inputWrites;
  receipt.observations = observations;
  const retain = () => fs.writeFileSync(output, JSON.stringify(receipt, null, 2) + "\n");
  flush = retain;
  retain();
  const native = observed(createRequire(import.meta.url)(path.resolve(binding)) as Native);
  receipt.source = sourceIdentity(binding, native);
  retain();
  const projectRoot = path.join(path.dirname(output), path.basename(output, ".json"));
  const packet = captureImportedTypeProjectStats(native, projectRoot + "-project");
  receipt.stats = packet;
  retain();
  const unresolved = captureUnresolvedImportedTypeStats(native, projectRoot + "-unresolved");
  receipt.unresolved = unresolved;
  retain();
  assertImportedTypeProjectStats({ ...packet, unresolved });
  receipt.updates = captureProjectUpdates(native, projectRoot + "-updates");
  receipt.status = "qualified";
  retain();
}
