import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";
import { gunzipSync } from "node:zlib";

const root = new URL("../_fixtures/reference/vue1/", import.meta.url);
const receipt = JSON.parse(readFileSync(new URL("receipt.json", root), "utf8"));
const gzip = readFileSync(new URL("vue.common.cjs.gz", root));
const source = gunzipSync(gzip);
const sha256 = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
assert.equal(gzip.length, receipt.gzipBytes);
assert.equal(sha256(gzip), receipt.gzipSha256);
assert.equal(source.length, receipt.sourceBytes);
assert.equal(sha256(source), receipt.sourceSha256);
assert.equal(
  createHash("sha1").update(`blob ${source.length}\0`).update(source).digest("hex"),
  "63ebe385546dec4073dacbe13878d3f6a1759053",
);
const module = { exports: {} };
vm.runInNewContext(
  source.toString("utf8"),
  {
    module,
    exports: module.exports,
    process: { env: { NODE_ENV: "test" } },
    console,
    setTimeout: () => 0,
  },
  { timeout: 1000, filename: "actual-pinned-vue-1.0.28.cjs" },
);
const Vue = module.exports as {
  version: string;
  config: { silent: boolean };
  options: object;
  compiler: { compile(node: object, options: object, partial: boolean): unknown };
  parsers: {
    text: { parseText(source: string): unknown; tokensToExp(tokens: unknown): string };
    directive: { parseDirective(source: string): unknown };
  };
};
assert.equal(Vue.version, "1.0.28");
Vue.config.silent = true;
const parse = (value: string) => JSON.parse(JSON.stringify(Vue.parsers.text.parseText(value)));

test("the pinned actual Vue1 parser proves raw and escaped text framing", () => {
  const token = (value: string, html: boolean, oneTime = false) => ({
    tag: true,
    value,
    html,
    oneTime,
  });
  for (const [value, expected] of [
    ["{{ x }}", [token("x", false)]],
    ["{{{ x }}}", [token("x", true)]],
    ["{{{ x }}", [token("{ x", false)]],
    ["{{{ x }}abc", [token("{ x", false), { value: "abc" }]],
    ["{{{a}}b}}}", [token("a}}b", true)]],
    ["{{{}}}", [token("{", false), { value: "}" }]],
    ["{{{ }}}", [token("", true)]],
    ["{{}}", null],
    ["{{ a\r b }}", null],
    ["{{{ a\u2028b }}}", null],
    ["{{{ a\u2029b }}}", null],
    ["{{ a\n b }}", [token("a\n b", false)]],
    ["{{{* x }}}", [token("x", true, true)]],
    ["{{{ * x }}}", [token("* x", true)]],
    ["{{{{x}}}}", [token("{x", true), { value: "}" }]],
    ["{{{a}}}{{{b}}}", [token("a", true), token("b", true)]],
    [
      "a {{{ 雪 }}} {{x}} z",
      [{ value: "a " }, token("雪", true), { value: " " }, token("x", false), { value: " z" }],
    ],
  ] as const) {
    assert.deepEqual(parse(value), expected, value);
  }
});

test("the actual Vue1 compiler suppresses children only for a literal pre attribute", () => {
  for (const name of ["v-pre", "v-pre.foo", "v-pre:arg", "v-pre:[broken", "@pre", "v-previous"]) {
    let childrenRead = false;
    const attributes = [{ name, value: "" }];
    // This minimal DOM contract records the actual compiler's child traversal.
    const node = {
      nodeType: 1,
      tagName: "DIV",
      attributes,
      getAttribute(key: string) {
        return attributes.find((a) => a.name === key)?.value ?? null;
      },
      removeAttribute(key: string) {
        const index = attributes.findIndex((a) => a.name === key);
        if (index >= 0) attributes.splice(index, 1);
      },
      hasAttribute(key: string) {
        return attributes.some((a) => a.name === key);
      },
      hasAttributes() {
        return attributes.length > 0;
      },
      hasChildNodes() {
        return true;
      },
      get childNodes() {
        childrenRead = true;
        return [];
      },
    };
    Vue.compiler.compile(node, Vue.options, true);
    assert.equal(childrenRead, name !== "v-pre", name);
  }
});

test("actual Vue1 text and directive owners retain the bounded expression spellings", () => {
  for (const [source, expression] of [
    ["{{ value }}", "value"],
    ["{{ user.name }}", "user.name"],
    ["{{ left + right }}", "left + right"],
    ["{{ ready ? yes : no }}", "ready ? yes : no"],
    ["{{ call(value, 2) }}", "call(value, 2)"],
    ["{{ [first, next] }}", "[first, next]"],
    ["{{ /*keep*/ msg && 条件 }}", "/*keep*/ msg && 条件"],
    ["{{\n value\n}}", "value"],
    ["{{\u00a0\ufeffvalue\u3000}}", "value"],
    ["{{ '&#42;' }}", "'&#42;'"],
    ["{{ /*kept*/ value + }}", "/*kept*/ value +"],
  ]) {
    const tokens = parse(source);
    assert.deepEqual(tokens, [{ tag: true, value: expression, html: false, oneTime: false }]);
    assert.deepEqual(JSON.parse(JSON.stringify(Vue.parsers.directive.parseDirective(expression))), {
      expression,
    });
    assert.equal(Vue.parsers.text.tokensToExp(tokens), expression);
  }
  // These are historical framing/spelling oracles. They do not evaluate JS,
  // prove native AST admission, or substitute generated text for parser input.
});

test("actual Vue1 pipe grammar proves why the whole pipe family remains deferred", () => {
  for (const [source, expected] of [
    ["left || right", { expression: "left || right" }],
    ["'a|b'", { expression: "'a|b'" }],
    ["value | upper", { expression: "value", filters: [{ name: "upper" }] }],
    [
      "value | upper 'arg' flag 2",
      {
        expression: "value",
        filters: [
          {
            name: "upper",
            args: [
              { value: "arg", dynamic: false },
              { value: "flag", dynamic: true },
              { value: 2, dynamic: false },
            ],
          },
        ],
      },
    ],
  ] as const) {
    assert.deepEqual(
      JSON.parse(JSON.stringify(Vue.parsers.directive.parseDirective(source))),
      expected,
    );
  }
});

test("actual Vue1 decoded framing checks separators and the once marker before trim", () => {
  // Explicit already-decoded browser text inputs: the original Vue1 parser
  // does not decode HTML. Native decode-map laws independently prove one decode.
  for (const source of ["{{\r x }}", "{{ a\r\n b }}", "{{ x \u2028}}", "{{ x \u2029}}"])
    assert.equal(parse(source), null, source);
  for (const [source, value, oneTime] of [
    ["{{* x }}", "x", true],
    ["{{ * x }}", "* x", false],
    ["{{ }}", "", false],
    ["{{ '雪' }}", "'雪'", false],
  ] as const)
    assert.deepEqual(parse(source), [{ tag: true, value, html: false, oneTime }]);
  assert.deepEqual(parse("{{ '}}' }}"), [
    { tag: true, value: "'", html: false, oneTime: false },
    { value: "' }}" },
  ]);
});
