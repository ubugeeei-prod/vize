import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
const hash = (b) => createHash("sha256").update(b).digest("hex");
const file = (p) => {
  const b = fs.readFileSync(p);
  return { path: fs.realpathSync(p), bytes: b.length, sha256: hash(b), body: b.toString("utf8") };
};
const req = createRequire(path.join(process.argv[2], "tests/package.json"));
function inspect(name, from = req) {
  const entry = fs.realpathSync(from.resolve(name));
  let root = path.dirname(entry);
  while (true) {
    const p = path.join(root, "package.json");
    if (fs.existsSync(p)) {
      const m = JSON.parse(fs.readFileSync(p, "utf8"));
      if (m.name === name)
        return {
          name,
          root,
          entry,
          entrySha256: file(entry).sha256,
          version: m.version,
          manifest: file(p),
          declaration: m.types ? file(path.resolve(root, m.types)) : null,
        };
    }
    const parent = path.dirname(root);
    if (parent === root) throw Error("no package owner: " + name);
    root = parent;
  }
}
const vue = inspect("vue");
const dom = inspect("@vue/runtime-dom", createRequire(vue.entry));
const core = inspect("@vue/runtime-core", createRequire(dom.entry));
const reactivity = inspect("@vue/reactivity", createRequire(dom.entry));
const typescript = inspect("typescript");
const vueTsc = inspect("vue-tsc");
const languageCore = inspect("@vue/language-core", createRequire(vueTsc.entry));
console.log(
  JSON.stringify({
    vue,
    dom,
    core,
    reactivity,
    typescript,
    vueTsc,
    languageCore,
    node: {
      version: process.version,
      executable: fs.realpathSync(process.execPath),
      sha256: hash(fs.readFileSync(process.execPath)),
    },
    scope:
      "actual workspace distribution bodies; native loaded-file membership and historical Misskey declarations UNOBSERVED",
  }),
);
