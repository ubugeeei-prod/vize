/** The actual emitted raw helper must retain its pre-ownership context contract. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

type ProcessResult = {
  status: number | null;
  signal: unknown;
  error: unknown;
  stdout: string;
  stderr: string;
};
type Run = (argv: string[], cwd: string, output: string) => ProcessResult;

const rawContext = `const raw = __vizeComponentAriaPropEntries('x', 'y', 123);
const exact: 123 = raw.x;
const contextShouldReject: {x: string; y: string} = __vizeComponentAriaPropEntries('x', 'y', 123);
const literal: {x: 'ok'; y: 'ok'} = __vizeComponentAriaPropEntries('x', 'y', 'ok');
declare function checkProps(value: {x: 'ok'; y: 'ok'}): void;
checkProps({ ...__vizeComponentAriaPropEntries('x', 'y', 123) });
declare function generic<T extends string>(value: {x:T; y:T}): T;
const genericValue = generic({ ...__vizeComponentAriaPropEntries('x', 'y', 123) });
`;
const reference =
  "declare function __vizeComponentAriaPropEntries<const R extends keyof any, const C extends keyof any, const T>(raw: R, camel: C, value: T): { readonly [K in R | C]: T extends undefined ? never : T };";

export function verifyRawAriaContext(output: string, host: string, run: Run) {
  const transforms = JSON.parse(
      fs.readFileSync(path.join(output, "projects/original/valid/transforms.json"), "utf8"),
    ),
    helper = (transforms[1].text as string)
      .split("\n")
      .filter((line) => line.startsWith("  function __vizeComponentAriaPropEntries"));
  assert.ok(helper.length > 0, "retain the actual emitted helper declarations");
  const prelude = "type $NotUndefined<T> = T extends undefined ? never : T;\n",
    referenceHeader = [reference, ...Array<string>(helper.length - 1).fill("")].join("\n"),
    programs = {
      reference: `${prelude}${referenceHeader}\n${rawContext}`,
      actual: `${prelude}${helper.join("\n")}\n${rawContext}`,
    },
    results: ProcessResult[] = [];
  for (const [name, program] of Object.entries(programs)) {
    const directory = path.join(output, "raw-context", name);
    fs.mkdirSync(directory, { recursive: true });
    fs.writeFileSync(path.join(directory, "context.ts"), program);
    fs.writeFileSync(
      path.join(directory, "tsconfig.json"),
      JSON.stringify({
        compilerOptions: { strict: true, noEmit: true, types: [], lib: ["esnext"] },
        files: ["context.ts"],
      }) + "\n",
    );
    results.push(
      run([host, "--noEmit", "--pretty", "false", "-p", "tsconfig.json"], directory, directory),
    );
  }
  const [before, actual] = results;
  assert.equal(typeof before.status, "number");
  assert.notEqual(before.status, 0);
  assert.equal(before.signal, null);
  assert.equal(before.error, null);
  assert.equal(before.stderr, "");
  assert.deepEqual(
    [...before.stdout.matchAll(/error TS(\d+):/gu)].map((match) => match[1]),
    ["2322", "2345", "2345"],
  );
  for (const key of ["status", "signal", "error", "stdout", "stderr"] as const)
    assert.deepEqual(actual[key], before[key], `raw helper context changed ${key}`);
  return { passed: true, referenceErrors: 3, helperDeclarations: helper.length };
}
