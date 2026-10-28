/** Preserve complete compiler messages and independently query stock TypeScript types. */
import assert from "node:assert/strict";
import path from "node:path";
import { createRequire } from "node:module";
import { identify, save } from "./canon-instance-directives-io.ts";
import { isRecord } from "./canon-instance-directives-model.ts";
type ObjectValue = Record<string, unknown>;
function object(value: unknown): ObjectValue {
  assert.ok(isRecord(value));
  return value;
}
function call(owner: ObjectValue, key: string, args: unknown[]): unknown {
  const fn = owner[key];
  assert.ok(typeof fn === "function");
  const result: unknown = Reflect.apply(fn, owner, args);
  return result;
}
export function diagnostics(bytes: Buffer, status: number | null, root: string) {
  const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  const result: { file: string; line: number; column: number; code: number; message: string }[] =
    [];
  for (const line of text.split(/\r?\n/u)) {
    const match = /^(.+?)\((\d+),(\d+)\): error TS(\d+): (.*)$/u.exec(line);
    if (match) {
      const [, file, line, column, code, message] = match;
      assert.ok(file && line && column && code && message !== undefined);
      result.push({
        file: path.relative(root, path.resolve(root, file)).replaceAll("\\", "/"),
        line: +line,
        column: +column,
        code: +code,
        message,
      });
    } else if (line.length) {
      const previous = result.at(-1);
      assert.ok(previous, "non-diagnostic output");
      previous.message += "\n" + line;
    }
  }
  assert.equal(result.length, (text.match(/error TS\d+:/gu) ?? []).length);
  assert.ok(status === (result.length ? 1 : 0) || status === (result.length ? 2 : 0));
  return result;
}
export function stockCalibration(provider: string, file: string, out: string) {
  const req = createRequire(path.join(provider, "package.json")),
    ts = object(req("typescript"));
  const program = object(
    call(ts, "createProgram", [
      {
        rootNames: [file],
        options: { strict: true, noEmit: true, skipLibCheck: true, types: [], target: 99 },
      },
    ]),
  );
  const checker = object(call(program, "getTypeChecker", [])),
    source = object(call(program, "getSourceFile", [file]));
  const rows: ObjectValue[] = [];
  function visit(value: unknown): void {
    const node = object(value);
    if (call(ts, "isIdentifier", [node])) {
      const name = node.text;
      if (
        typeof name === "string" &&
        /^(?:error|any|unknown|never|number)(?:Value|Result)$/u.test(name) &&
        !rows.some((row) => row.name === name)
      ) {
        const type = object(call(checker, "getTypeAtLocation", [node]));
        rows.push({
          name,
          type: call(checker, "typeToString", [type]),
          flags: type.flags,
          intrinsicName: type.intrinsicName ?? null,
          id: type.id,
        });
      }
    }
    call(ts, "forEachChild", [node, visit]);
  }
  visit(source);
  save(path.join(out, "stock-calibration.json"), {
    compiler: identify(req.resolve("typescript/lib/typescript.js")),
    rows,
  });
}
