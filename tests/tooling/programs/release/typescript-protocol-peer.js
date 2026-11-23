const fs = require("node:fs"),
  path = require("node:path");
const args = process.argv.slice(2),
  fault = process.env.VIZE_SMOKE_RUNTIME_TEST_FAULT;
fs.appendFileSync(
  process.env.VIZE_SMOKE_RUNTIME_TEST_LOG,
  JSON.stringify({ tool: "tsgo", args, cwd: process.cwd() }) + "\n",
);
if (args.includes("tsconfig.error.json")) {
  if (fault === "false-clean") process.exit(0);
  const diagnostics = [
    ["errors/Broken.vue", "TS2322"],
    ["errors/JavaScriptConsumer.js", "TS2322"],
    ["errors/JsxConsumer.jsx", "TS2322"],
    ["src/Unused.vue", "TS6133"],
  ];
  diagnostics.forEach(([file, code], index) => {
    if (fault === "drop-" + index) return;
    const text =
      fault === "split-diagnostic" && index === 0 ? file + "\n" + code : file + " " + code;
    (index % 2 === 0 ? process.stdout : process.stderr).write(text + "\n");
  });
  process.exit(1);
}
if (args.includes("tsconfig.emit.json")) {
  fs.mkdirSync("dist", { recursive: true });
  if (fault !== "missing-app")
    fs.writeFileSync(
      path.join("dist", fault === "alternate-declaration" ? "App.d.vue.ts" : "App.vue.d.ts"),
      "export {}\n",
    );
  if (fault !== "missing-main") fs.writeFileSync("dist/main.d.ts", "export {}\n");
}
