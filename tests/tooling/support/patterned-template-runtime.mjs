import assert from "node:assert/strict";
import process from "node:process";
import { transformSync } from "@babel/core";
import { Window } from "happy-dom";
import { traceMountedBackend } from "./davinci-mounted-trace.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const { backend, code, cases } = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.ok(["vdom", "vapor", "ssr"].includes(backend));

// Resolve real SSR runtime imports without modifying any generated expressions.
let ssrRender;
if (backend === "ssr") {
  const compiled = transformSync(code, {
    configFile: false,
    babelrc: false,
    plugins: [
      () => ({
        visitor: {
          ImportDeclaration(path) {
            assert.ok(["vue", "@vue/server-renderer"].includes(path.node.source.value));
            path.node.source.value = import.meta.resolve(
              path.node.source.value === "@vue/server-renderer" ? "vue/server-renderer" : "vue",
            );
          },
        },
      }),
    ],
  }).code;
  ({ ssrRender } = await import(
    `data:text/javascript;base64,${Buffer.from(`${compiled}\nexport { ssrRender };`).toString("base64")}`
  ));
}

for (const fixture of cases) {
  let reads = 0;
  const context = { ...fixture.context };
  switch (fixture.scenario) {
    case undefined:
      break;
    case "own-undefined":
      context.subject = { field: undefined };
      break;
    case "inherited-undefined":
      context.subject = Object.create({ field: undefined });
      break;
    case "inherited-getter":
      context.subject = Object.create({
        get field() {
          reads++;
          return 1;
        },
      });
      break;
    case "short-array-getter": {
      context.subject = [1];
      Object.defineProperty(context.subject, "0", {
        get() {
          reads++;
          return 1;
        },
      });
      break;
    }
    default:
      throw new Error(`unknown scenario: ${fixture.scenario}`);
  }
  const steps = fixture.steps ?? [];
  let trees;
  if (backend === "ssr") {
    const window = new Window();
    try {
      trees = [];
      for (const step of [{ patch: {} }, ...steps]) {
        Object.assign(context, step.patch);
        let html = "";
        ssrRender(
          context,
          (part) => {
            html += part;
          },
          null,
          {},
        );
        window.document.body.innerHTML = html;
        trees.push(observe(window.document.body));
      }
    } finally {
      await window.happyDOM.close();
    }
  } else {
    const trace = await traceMountedBackend({ backend, code, context, steps });
    assert.deepEqual(trace.at(-1), { tree: [], events: [] });
    trees = trace.slice(0, -1).map(({ tree, events }) => {
      assert.deepEqual(events, []);
      return tree;
    });
  }
  assert.deepEqual(trees, fixture.trees, `${backend}: ${JSON.stringify(fixture)}`);
  assert.equal(reads, fixture.reads ?? 0, "unexpected property reads");
}
process.stdout.write(JSON.stringify({ passed: cases.length }));

function observe(parent) {
  const children = [];
  for (const node of parent.childNodes) {
    if (node.nodeType === 3 && node.data) {
      if (typeof children.at(-1) === "string") children[children.length - 1] += node.data;
      else children.push(node.data);
    } else if (node.nodeType === 1) {
      children.push({
        tag: node.localName,
        attributes: Object.fromEntries(
          [...node.attributes].map(({ name, value }) => [name, value]),
        ),
        children: observe(node),
      });
    }
  }
  return children;
}
