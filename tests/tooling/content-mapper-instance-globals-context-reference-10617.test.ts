import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import {
  createTs59ContextReferences,
  isFlatTs59Context,
  type Ts59ContextInputs,
} from "./support/content-mapper-instance-globals-context-reference-10617.ts";
import type { Packet } from "./support/content-mapper-instance-globals-10617.ts";

const fixture = new URL(
  "../fixtures/typechecker/undeclared-instance-globals-10617/",
  import.meta.url,
);
const cases = JSON.parse(fs.readFileSync(new URL("cases.json", fixture), "utf8")).cases as Array<{
  id: string;
  globals: string;
  oracle: string;
}>;
const config = JSON.parse(fs.readFileSync(new URL("project/tsconfig.json.txt", fixture), "utf8"));
delete config.contentMappers;
delete config.include;
config.files = ["src/globals.d.ts", "oracle/Context.ts"];
function input(entry = cases[0]): Ts59ContextInputs {
  return {
    command: "/runtime/node",
    args: [
      "/reference/typescript/bin/tsc",
      "--noEmit",
      "--pretty",
      "false",
      "-p",
      "oracle.tsconfig.json",
    ],
    cwd: "/source",
    config: JSON.stringify(config),
    globals: entry.globals,
    context: entry.oracle,
  };
}
function packet(label: string): Packet {
  return {
    label,
    command: "/runtime/node",
    args: input().args,
    cwd: "/source",
    status: 2,
    signal: null,
    error: null,
    stdout: "oracle/Context.ts(1,1): error TS2339: authored diagnostic\n",
    stderr: "",
    elapsedMs: 10,
  };
}

test("all forty fixture contexts are flat and reuse only their own genuine first packet", () => {
  assert.equal(cases.length, 40);
  const reference = createTs59ContextReferences();
  const processes: Packet[] = [];
  const originals = new Map<string, Packet>();
  for (const variant of ["lf", "crlf-unicode"]) {
    for (const entry of cases) {
      const inputs = input(entry);
      assert.equal(isFlatTs59Context(inputs), true, entry.id);
      const result = reference(entry.id, inputs, () => {
        const result: Packet = packet(`${variant}/${entry.id}`);
        processes.push(result);
        return result;
      });
      if (variant === "lf") originals.set(entry.id, result);
      assert.strictEqual(result, originals.get(entry.id));
      assert.equal(result.label, `lf/${entry.id}`, "references never pretend another process ran");
    }
  }
  assert.equal(processes.length, 40);
  assert.equal(new Set(processes).size, 40, "negative/repair cases retain independent attempts");
});

test("every command, provider, cwd and project byte change requires another real attempt", () => {
  const original = input();
  const mutations: Array<(value: Ts59ContextInputs) => void> = [
    (value) => {
      value.command += "-other";
    },
    (value) => {
      value.args[0] += "-other";
    },
    (value) => {
      value.cwd += "-other";
    },
    (value) => {
      value.config += "\n";
    },
    (value) => {
      value.globals += "\n";
    },
    (value) => {
      value.context += "\n";
    },
  ];
  for (const mutate of mutations) {
    const reference = createTs59ContextReferences();
    const first = reference("original", original, () => packet("first"));
    const changed = structuredClone(original);
    mutate(changed);
    const second = reference("original", changed, () => packet("changed"));
    assert.notStrictEqual(second, first);
    assert.equal(second.label, "changed");
  }
});

test("unproven imports, references, compiler options and commands always execute", () => {
  const mutations: Array<(value: Ts59ContextInputs) => void> = [
    (value) => {
      value.globals += '\nimport "./Globals.vue";';
    },
    (value) => {
      value.context += '\ntype Other = import("./Globals.vue");';
    },
    (value) => {
      value.context += '\n/// <reference path="../src/Globals.vue" />';
    },
    (value) => {
      value.context += '\nexport { Other } from "./other";';
    },
    (value) => {
      value.context += '\nimport { Other } from "other";';
    },
    (value) => {
      value.context += '\nconst x = require("./other");';
    },
    (value) => {
      value.config = JSON.stringify({ ...config, include: ["src"] });
    },
    (value) => {
      value.config = JSON.stringify({
        ...config,
        compilerOptions: { ...config.compilerOptions, paths: { vue: ["src/Globals.vue"] } },
      });
    },
    (value) => {
      value.config = "{";
    },
    (value) => {
      value.args.push("--incremental");
    },
    (value) => {
      value.command = "node";
    },
  ];
  for (const mutate of mutations) {
    const reference = createTs59ContextReferences();
    const changed = input();
    mutate(changed);
    assert.equal(isFlatTs59Context(changed), false);
    const first = reference("original", changed, () => packet("first"));
    const second = reference("original", changed, () => packet("second"));
    assert.notStrictEqual(first, second);
  }
});

test("unproven input invalidates earlier references and failed processes are never reused", () => {
  const reference = createTs59ContextReferences();
  const original = input();
  const first = reference("original", original, () => packet("first"));
  reference("other", { ...original, context: 'import "other";' }, () => packet("unknown"));
  assert.notStrictEqual(
    reference("original", original, () => packet("after unknown")),
    first,
  );
  for (const changes of [
    { status: null },
    { status: 1 },
    { error: "spawn failed" },
    { signal: "SIGTERM" },
    { stderr: "unexpected" },
  ]) {
    const reference = createTs59ContextReferences();
    const first = reference("original", original, () => ({ ...packet("failed"), ...changes }));
    const second = reference("original", original, () => packet("second"));
    assert.notStrictEqual(first, second);
  }
});
