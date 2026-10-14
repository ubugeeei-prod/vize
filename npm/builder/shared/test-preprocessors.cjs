// Test-only stand-ins for the sass, less and stylus packages, installed into a
// temporary project's node_modules. They implement the subset the bundler
// integrations call and the subset of syntax the fixtures use: nested rules
// with `&`, `$name: value;` variables, and Sass/Stylus indentation. Nesting
// is flattened the way the real preprocessors flatten it, so scoping applied
// before them would produce descendant selectors such as `[data-v-x] .root`.
"use strict";

const fs = require("node:fs");
const path = require("node:path");

function flatten(source, parent) {
  const declarations = [];
  const nested = [];
  let text = "";
  let index = 0;
  while (index < source.length) {
    const char = source[index];
    if (char === "{") {
      let depth = 1;
      let end = index + 1;
      while (end < source.length && depth > 0) {
        if (source[end] === "{") depth += 1;
        if (source[end] === "}") depth -= 1;
        end += 1;
      }
      const selector = text.trim();
      const full = !parent
        ? selector
        : selector.includes("&")
          ? selector.replaceAll("&", parent)
          : `${parent} ${selector}`;
      nested.push(...flatten(source.slice(index + 1, end - 1), full));
      text = "";
      index = end;
      continue;
    }
    if (char === ";") {
      if (text.trim()) declarations.push(`${text.trim()};`);
      text = "";
    } else {
      text += char;
    }
    index += 1;
  }
  if (text.trim()) declarations.push(`${text.trim()};`);
  const own = parent && declarations.length > 0 ? [`${parent} { ${declarations.join(" ")} }`] : [];
  return [...own, ...nested];
}

/** Indentation syntax (Sass, Stylus) to braces. */
function indentedToBraces(source) {
  const lines = source.split("\n").filter((line) => line.trim());
  let output = "";
  const stack = [];
  lines.forEach((line, index) => {
    const indent = line.length - line.trimStart().length;
    while (stack.length > 0 && stack.at(-1) >= indent) {
      stack.pop();
      output += "}\n";
    }
    const next = lines[index + 1];
    const nextIndent = next ? next.length - next.trimStart().length : -1;
    if (nextIndent > indent) {
      output += `${line.trim()} {\n`;
      stack.push(indent);
    } else {
      output += `${line.trim().replace(/^([\w-]+)\s+(?!:)/, "$1: ")};\n`;
    }
  });
  return output + "}\n".repeat(stack.length);
}

function compile(source, indented = false) {
  const variables = new Map();
  const braced = indented ? indentedToBraces(source) : source;
  const body = braced.replace(/\$([\w-]+)\s*:\s*([^;]+);/g, (_, name, value) => {
    variables.set(name, value.trim());
    return "";
  });
  const substituted = body.replace(/\$([\w-]+)/g, (match, name) => variables.get(name) ?? match);
  return flatten(substituted, "").join("\n");
}

const PACKAGES = {
  sass: `const { compile } = require(STAND_IN);
const run = (source, options = {}) => ({
  css: compile(source, options.syntax === "indented"),
  loadedUrls: [],
});
exports.info = "dart-sass\\t1.80.0";
exports.compileString = run;
exports.compileStringAsync = async (source, options) => run(source, options);
exports.initAsyncCompiler = async () => ({
  compileStringAsync: async (source, options) => run(source, options),
  dispose() {},
});
`,
  less: `const { compile } = require(STAND_IN);
exports.FileManager = class FileManager {};
exports.render = async (source) => ({ css: compile(source), imports: [] });
`,
  stylus: `const { compile } = require(STAND_IN);
module.exports = (source) => {
  const ref = {
    set: () => ref,
    define: () => ref,
    include: () => ref,
    import: () => ref,
    deps: () => [],
    render: (callback) => {
      const css = compile(source, !source.includes("{"));
      return callback ? callback(null, css) : css;
    },
  };
  return ref;
};
`,
};

/** Write sass, less and stylus stand-ins into `<root>/node_modules`. */
function installStandIns(root) {
  fs.writeFileSync(path.join(root, "package.json"), '{ "name": "fixture", "private": true }\n');
  for (const [name, source] of Object.entries(PACKAGES)) {
    const dir = path.join(root, "node_modules", name);
    fs.mkdirSync(dir, { recursive: true });
    fs.writeFileSync(
      path.join(dir, "package.json"),
      `{ "name": "${name}", "version": "99.0.0", "main": "index.cjs" }\n`,
    );
    fs.writeFileSync(
      path.join(dir, "index.cjs"),
      source.replace("STAND_IN", JSON.stringify(__filename)),
    );
  }
}

module.exports = { compile, installStandIns };
