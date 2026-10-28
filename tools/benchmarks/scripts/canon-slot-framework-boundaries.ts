/** Additional ownership boundaries are observations, never a policy waiver. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { identify, save } from "./canon-instance-directives-io.ts";
import { isRecord } from "./canon-instance-directives-model.ts";
type Case = { id: string; component: string; source: string; requiresAuthoredError: boolean };
export function boundaryCases(zero: Case): Case[] {
  const signature = "new(): Instance";
  assert.equal(zero.component.split(signature).length, 2);
  const rows: Case[] = [];
  const add = (id: string, component: string, source: string, repairComponent = component) => {
    rows.push({
      id: "boundary-" + id + "-invalid",
      component,
      source,
      requiresAuthoredError: false,
    });
    rows.push({
      id: "boundary-" + id + "-repaired",
      component: repairComponent,
      source: zero.source,
      requiresAuthoredError: false,
    });
  };
  add(
    "required-ctor",
    zero.component.replace(signature, "new(props: { value: number }): Omit<Instance, '$props'>"),
    zero.source.replace(' :value="1"', ""),
  );
  add(
    "absent-props",
    zero.component.replace(
      signature,
      "new(props: { value: number }, token: string): Pick<Instance, '$slots'>",
    ),
    zero.source.replace(':value="1"', ":value=\"'bad'\""),
  );
  add(
    "unknown-provider",
    zero.component.replace("{ new(): Instance }", "unknown"),
    zero.source,
    zero.component,
  );
  add(
    "library-fallback",
    zero.component.replace(
      signature,
      "new(props: { value: number; internal: string }): Instance; (props: { value: number }): { __ctx: { props: { value: number }; slots: Instance['$slots'] } }",
    ),
    zero.source.replace(':value="1"', ":value=\"'bad'\""),
  );
  return rows;
}
function record(value: unknown): Record<string, unknown> {
  assert.ok(isRecord(value));
  return value;
}
function array(value: unknown): unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}
function string(value: unknown): string {
  assert.equal(typeof value, "string");
  assert.ok(typeof value === "string");
  return value;
}
function numeric(value: unknown): number {
  assert.ok(typeof value === "number" && Number.isSafeInteger(value));
  return value;
}
function call(owner: Record<string, unknown>, method: string, args: unknown[]): unknown {
  const fn = owner[method];
  assert.ok(typeof fn === "function");
  const result: unknown = Reflect.apply(fn, owner, args);
  return result;
}
function fullDiagnostics(file: string, project: string) {
  const text = new TextDecoder("utf8", { fatal: true }).decode(fs.readFileSync(file));
  const rows: { file: string; line: number; column: number; code: number; message: string }[] = [];
  for (const line of text.split(/\r?\n/u)) {
    const m = /^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
    if (m) {
      const [, file, line, column, code, message] = m;
      assert.ok(file && line && column && code && message !== undefined);
      rows.push({
        file: path.relative(project, path.resolve(project, file)).replaceAll("\\", "/"),
        line: +line,
        column: +column,
        code: +code,
        message,
      });
    } else if (line.length) {
      const prior = rows.at(-1);
      assert.ok(prior, "unexpected non-diagnostic stdout");
      prior.message += "\n" + line;
    }
  }
  assert.equal(rows.length, (text.match(/error TS\d+:/gu) ?? []).length);
  return rows;
}
export function boundaryEvidence(project: string, provider: string, out: string, rows: Case[]) {
  const selected = rows.filter((row) => row.id.startsWith("boundary-"));
  const source = ["import {h,type Component} from 'vue';"];
  for (const [index, row] of selected.entries())
    source.push(
      "import C" + index + " from './" + row.id + "/component';",
      "const c" + index + ":Component=C" + index + ";",
      "h(C" + index + ",{value:1});void c" + index + ";",
    );
  const file = path.join(project, "boundary_h.ts");
  fs.writeFileSync(file, source.join("\n") + "\n");
  const req = createRequire(path.join(provider, "package.json")),
    ts = record(req("typescript"));
  const options = {
    strict: true,
    skipLibCheck: true,
    noEmit: true,
    types: [],
    target: numeric(record(ts.ScriptTarget).ESNext),
    module: numeric(record(ts.ModuleKind).ESNext),
    moduleResolution: numeric(record(ts.ModuleResolutionKind).Bundler),
  };
  const program = record(call(ts, "createProgram", [{ rootNames: [file], options }]));
  const diagnostics = array(call(ts, "getPreEmitDiagnostics", [program])).map((item) => {
    const d = record(item),
      f = d.file;
    let point: { file: string; line: number; column: number } | null = null;
    if (f !== undefined) {
      const owned = record(f),
        at = record(call(owned, "getLineAndCharacterOfPosition", [numeric(d.start)]));
      point = {
        file: path.relative(project, string(owned.fileName)).replaceAll("\\", "/"),
        line: numeric(at.line) + 1,
        column: numeric(at.character) + 1,
      };
    }
    return {
      ...point,
      code: numeric(d.code),
      category: numeric(d.category),
      message: string(call(ts, "flattenDiagnosticMessageText", [d.messageText, "\n"])),
    };
  });
  const raw = [
    { role: "official", file: "official-vue-tsc.stdout.bin", project },
    { role: "published-mapper", file: "published-mapper.stdout.bin", project },
    {
      role: "candidate-mapper",
      file: "source-candidate-mapper.stdout.bin",
      project: path.join(out, "source-candidate"),
    },
  ].map((item) => ({
    ...item,
    diagnostics: fullDiagnostics(path.join(out, item.file), item.project).filter((d) =>
      d.file.startsWith("boundary-"),
    ),
  }));
  const evidence = {
    schema: 1,
    scope:
      "four boundary domains, invalid/repair pairs; full multiline diagnostic messages; no production policy qualification",
    inputs: selected,
    stock: {
      compiler: identify(req.resolve("typescript/lib/typescript.js")),
      source: source.join("\n") + "\n",
      diagnostics,
    },
    raw,
    timingUsable: false,
    policyQualified: false,
  };
  save(path.join(out, "boundary-summary.json"), evidence);
  return evidence;
}
