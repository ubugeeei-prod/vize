const assert = require("node:assert/strict");
const { compileFunction } = require("node:vm");
let code = "";
process.stdin.setEncoding("utf8");
process.stdin.on("data", (chunk) => (code += chunk));
process.stdin.on("end", () => {
  const vnode = (_type, props) => props;
  const Vue = {
    openBlock() {},
    resolveComponent: (name) => name,
    createBlock: vnode,
    createVNode: vnode,
    createElementBlock: vnode,
    createElementVNode: vnode,
  };
  const render = compileFunction(code + "\nreturn render;", ["Vue", "isInPopOutWindow"]);
  for (const value of [false, true]) {
    const output = render(Vue, value)({}, [], {}, {}, {}, {});
    assert.deepEqual(output, { teleported: !value, after: 42 });
  }
  process.stdout.write("ok\n");
});
