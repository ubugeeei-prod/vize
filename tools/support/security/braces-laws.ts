import assert from "node:assert/strict";

export type BracesOptions = Record<string, unknown>;
export type BracesAst = { type: string; value?: string; nodes?: BracesAst[] };
export type Braces = ((input: string, options?: BracesOptions) => string[]) & {
  parse(input: string, options?: BracesOptions): BracesAst;
  compile(input: string | BracesAst, options?: BracesOptions): string;
  expand(input: string | BracesAst, options?: BracesOptions): string[];
  stringify(input: string | BracesAst, options?: BracesOptions): string;
};

export function nestedAst(depth: number): BracesAst {
  let node: BracesAst = { type: "text", value: "a" };
  for (let i = 0; i < depth; i++) node = { type: "brace", nodes: [node] };
  return { type: "root", nodes: [node] };
}

export function verifyBracesDepth(braces: Braces): {
  negativeChecks: number;
  positiveChecks: number;
} {
  let negativeChecks = 0,
    positiveChecks = 0;
  const rejected = (call: () => unknown, kind: typeof SyntaxError | typeof RangeError) => {
    assert.throws(
      call,
      (error: unknown) => error instanceof kind && /exceeds max depth \(100\)/.test(error.message),
    );
    negativeChecks++;
  };
  const deep = "{".repeat(4000) + "a,b" + "}".repeat(4000);
  assert.ok(deep.length < 10000);
  for (const options of [
    {},
    { maxDepth: 101 },
    { maxDepth: 1e9 },
    { maxDepth: Infinity },
    { maxDepth: NaN },
  ]) {
    rejected(() => braces(deep, options), SyntaxError);
    for (const method of ["parse", "compile", "expand", "stringify"] as const)
      rejected(() => braces[method](deep, options), SyntaxError);
    rejected(() => braces.parse("(".repeat(101) + "a" + ")".repeat(101), options), SyntaxError);
    for (const method of ["compile", "expand", "stringify"] as const)
      rejected(() => braces[method](nestedAst(4000), options), RangeError);
  }
  const boundaries = [
    ["{".repeat(100) + "a,b" + "}".repeat(100), 100],
    ["(".repeat(100) + "a" + ")".repeat(100), 100],
    ["{(".repeat(50) + "a,b" + ")}".repeat(50), 100],
  ] as const;
  for (const [pattern] of boundaries) {
    assert.doesNotThrow(() => braces.compile(pattern));
    assert.doesNotThrow(() => braces.expand(pattern));
    assert.doesNotThrow(() => braces.stringify(pattern));
    positiveChecks += 3;
  }
  for (const method of ["compile", "stringify"] as const) {
    assert.equal(braces[method](nestedAst(100)), "a");
    positiveChecks++;
  }
  for (const method of ["parse", "compile", "expand", "stringify"] as const) {
    assert.throws(() => braces[method]("{{a,b},c}", { maxDepth: 1 }), /exceeds max depth \(1\)/);
    negativeChecks++;
    assert.doesNotThrow(() => braces[method]("{{a,b},c}", { maxDepth: 2 }));
    positiveChecks++;
    let reads = 0;
    const options: BracesOptions = {};
    Object.defineProperty(options, "maxDepth", { get: () => (++reads === 1 ? 100 : undefined) });
    rejected(() => braces[method](deep, options), SyntaxError);
    assert.equal(reads, 1);
  }
  for (const method of ["compile", "expand", "stringify"] as const) {
    let reads = 0;
    const options: BracesOptions = {};
    Object.defineProperty(options, "maxDepth", { get: () => (++reads === 1 ? 100 : undefined) });
    rejected(() => braces[method](nestedAst(4000), options), RangeError);
    assert.equal(reads, 1);
  }
  for (const method of ["compile", "expand", "stringify"] as const) {
    const cycle: BracesAst = { type: "root", nodes: [] };
    cycle.nodes!.push(cycle);
    rejected(() => braces[method](cycle), RangeError);
  }
  const escaped = "\\{".repeat(200) + "a" + "\\}".repeat(200);
  assert.doesNotThrow(() => braces.compile(escaped));
  positiveChecks++;
  const quoted = '"' + "{".repeat(200) + "a" + "}".repeat(200) + '"';
  assert.doesNotThrow(() => braces.compile(quoted));
  positiveChecks++;
  assert.deepEqual(braces.expand("src/{app,lib}/{1..3}.ts"), [
    "src/app/1.ts",
    "src/app/2.ts",
    "src/app/3.ts",
    "src/lib/1.ts",
    "src/lib/2.ts",
    "src/lib/3.ts",
  ]);
  positiveChecks++;
  return { negativeChecks, positiveChecks };
}
