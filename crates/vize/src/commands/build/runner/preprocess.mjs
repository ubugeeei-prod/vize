// Compile `<style lang="scss|sass|less|stylus">` blocks with the project's own
// preprocessor, resolved from each SFC's directory as a bundler would.
// stdin: [{ file, lang, source }]; stdout: one result per job, in order.
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname } from "node:path";
import { pathToFileURL } from "node:url";

const PACKAGES = {
  scss: ["sass", "sass-embedded"],
  sass: ["sass", "sass-embedded"],
  less: ["less"],
  stylus: ["stylus"],
  styl: ["stylus"],
};

function load(file, names) {
  const require = createRequire(file);
  for (const name of names) {
    try {
      const loaded = require(name);
      return loaded.default ?? loaded;
    } catch (error) {
      if (error?.code !== "MODULE_NOT_FOUND") throw error;
    }
  }
  return null;
}

async function compile(tool, { file, lang, source }) {
  const dir = dirname(file);
  if (lang === "scss" || lang === "sass") {
    return tool.compileString(source, {
      syntax: lang === "sass" ? "indented" : "scss",
      url: pathToFileURL(file),
      loadPaths: [dir],
    }).css;
  }
  if (lang === "less") {
    return (await tool.render(source, { filename: file, paths: [dir] })).css;
  }
  return tool(source).set("filename", file).include(dir).render();
}

/** Block-relative, 1-based position of a preprocessor error. */
function position(lang, error) {
  if (error?.span?.start) {
    return { line: error.span.start.line + 1, column: error.span.start.column + 1 };
  }
  if (lang === "less" && typeof error?.line === "number") {
    return { line: error.line, column: (error.column ?? 0) + 1 };
  }
  if (typeof error?.lineno === "number") {
    return { line: error.lineno, column: error.column ?? 1 };
  }
  return {};
}

const jobs = JSON.parse(readFileSync(0, "utf8"));
const results = [];
for (const job of jobs) {
  const names = PACKAGES[job.lang];
  try {
    const tool = load(job.file, names);
    results.push(tool ? { css: await compile(tool, job) } : { missing: names[0] });
  } catch (error) {
    const message = String(error?.sassMessage ?? error?.message ?? error).split("\n")[0];
    results.push({ error: { message, ...position(job.lang, error) } });
  }
}
process.stdout.write(JSON.stringify(results));
