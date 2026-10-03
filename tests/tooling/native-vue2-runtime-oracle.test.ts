import assert from "node:assert/strict";
import test from "node:test";
import { compiler, plain } from "./support/vue2-pinned-oracle.ts";
import { loadRuntime } from "./support/vue2-pinned-runtime.ts";

type Call = { name: string; args: unknown[] };
const normalized = (value: unknown): unknown => {
  if (Object.prototype.toString.call(value) === "[object RegExp]") {
    const regex = value as RegExp;
    return { regex: regex.source, flags: regex.flags };
  }
  return plain(value);
};
const cases: Array<[string, string, Call[]]> = [
  ["message | upper", "雪A", [{ name: "upper", args: ["雪a"] }]],
  [
    "value | add(2) | wrap('雪', '後')",
    "雪6後",
    [
      { name: "add", args: [4, 2] },
      { name: "wrap", args: [6, "雪", "後"] },
    ],
  ],
  ["left || right | upper", "3", [{ name: "upper", args: [3] }]],
  ["(left | right) | number", "3", [{ name: "number", args: [3] }]],
  ["'a|b' | upper", "A|B", [{ name: "upper", args: ["a|b"] }]],
  ['"a|b" | upper', "A|B", [{ name: "upper", args: ["a|b"] }]],
  ["`a|${value}` | upper", "A|4", [{ name: "upper", args: ["a|4"] }]],
  ["/a|b/.test(value) | number", "0", [{ name: "number", args: [false] }]],
  ["value / 2 | number", "2", [{ name: "number", args: [2] }]],
  ["list[index | mask] | number", "20", [{ name: "number", args: [20] }]],
  ["({a: value | mask}) | json", '{"a":5}', [{ name: "json", args: [{ a: 5 }] }]],
  [
    "value | pick(fn(1, 2), [3, 4], {a: 'x,y'}, /a,b/, `x,y`)",
    '[4,3,[3,4],{"a":"x,y"},{"regex":"a,b","flags":""},"x,y"]',
    [{ name: "pick", args: [4, 3, [3, 4], { a: "x,y" }, { regex: "a,b", flags: "" }, "x,y"] }],
  ],
  ["雪 | 日本語", "【冬】", [{ name: "日本語", args: ["冬"] }]],
  ["value | with-dash", "dash:4", [{ name: "withDash", args: [4] }]],
  ["value | upper()", "4", [{ name: "upper", args: [4] }]],
  ["value | upper ()", "SPACED:4", [{ name: "upper ", args: [4] }]],
  ["value", "4", []],
  ["value\r\n | upper", "4", [{ name: "upper", args: [4] }]],
  ["value &#124; add(2)", "6", [{ name: "add", args: [4, 2] }]],
  ["&#39;a|b&#39; | upper", "A|B", [{ name: "upper", args: ["a|b"] }]],
  ["&#160;value&#160; | add(2)", "6", [{ name: "add", args: [4, 2] }]],
];

const render = (environment: "test" | "production", template: string) => {
  const calls: Call[] = [];
  const filters = Object.fromEntries(
    Object.entries({
      upper: (value: unknown) => String(value).toUpperCase(),
      "upper ": (value: unknown) => `SPACED:${value}`,
      number: (value: unknown) => Number(value),
      add: (value: unknown, amount: unknown) => Number(value) + Number(amount),
      wrap: (value: unknown, before: unknown, after: unknown) => `${before}${value}${after}`,
      json: (value: unknown) => JSON.stringify(value),
      pick: (...values: unknown[]) => JSON.stringify(values.map(normalized)),
      日本語: (value: unknown) => `【${value}】`,
      withDash: (value: unknown) => `dash:${value}`,
    }).map(([name, implementation]) => [
      name,
      (...args: unknown[]) => {
        calls.push({ name, args: args.map(normalized) });
        return (implementation as (...values: unknown[]) => unknown)(...args);
      },
    ]),
  );
  const compiled = compiler.compile(template);
  assert.deepEqual(plain(compiled.errors), [], template);
  const executable = compiler.compileToFunctions(template);
  const Vue = loadRuntime(environment);
  const component = new Vue({
    data: () => ({
      message: "雪a",
      value: 4,
      left: 0,
      right: 3,
      mask: 1,
      index: 0,
      list: [10, 20],
      雪: "冬",
      fn: (a: number, b: number) => a + b,
      values: ["前", "後"],
    }),
    filters,
    ...executable,
  });
  const vnode = component._render();
  return {
    vnode: { tag: vnode.tag, children: Array.from(vnode.children, ({ text }) => ({ text })) },
    calls,
  };
};

test("actual official Vue2 runtime renders whole text values and ordered filter calls", () => {
  for (const environment of ["test", "production"] as const) {
    for (const [content, value, calls] of cases) {
      assert.deepEqual(
        render(environment, `<div>前 {{ ${content} }} 後</div>`),
        {
          vnode: { tag: "div", children: [{ text: `前 ${value} 後` }] },
          calls,
        },
        `${environment}: ${content}`,
      );
    }
  }
});

test("real runtime characterizes typed deferred families without native admission credit", () => {
  for (const environment of ["test", "production"] as const) {
    for (const [content, value, calls] of [
      ["value | wrap(...values)", "前4後", [{ name: "wrap", args: [4, "前", "後"] }]],
      ["value | add(2,)", "6", [{ name: "add", args: [4, 2] }]],
      ["/[a|b]/.test(value) | number", "0", [{ name: "number", args: [false] }]],
    ] as Array<[string, string, Call[]]>) {
      assert.deepEqual(render(environment, `<div>{{ ${content} }}</div>`), {
        vnode: { tag: "div", children: [{ text: value }] },
        calls,
      });
    }
    assert.deepEqual(render(environment, "<div>&#123;&#123; value &#125;&#125;</div>"), {
      vnode: { tag: "div", children: [{ text: "4" }] },
      calls: [],
    });
    assert.deepEqual(render(environment, "<div v-pre>前 {{ message | upper }} 後</div>"), {
      vnode: { tag: "div", children: [{ text: "前 {{ message | upper }} 後" }] },
      calls: [],
    });
  }
});
