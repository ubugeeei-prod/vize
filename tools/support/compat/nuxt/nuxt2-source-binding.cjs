const { createHash } = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const [root, binding, identity, source, ...args] = process.argv.slice(2);
const native = require(binding);
const selected = process.report
  .getReport()
  .sharedObjects.filter(
    (file) => file.endsWith(".node") && path.basename(file).includes("vize-vitrine"),
  );
if (selected.length !== 1) throw new Error("source Vize binding identity is ambiguous");
const file = fs.realpathSync(selected[0]);
if (!file.startsWith(root)) throw new Error("typecheck must load the current workspace binding");
fs.writeFileSync(
  identity,
  JSON.stringify(
    {
      path: file,
      sha256: createHash("sha256").update(fs.readFileSync(file)).digest("hex"),
      source,
    },
    null,
    2,
  ),
);
native.runCli(args);
