const { compile, inlineUses } = require(require("./stand-in-path.json"));
const run = (source, options = {}) => {
  try {
    const used = inlineUses(source, options.url);
    return { css: compile(used.css, options.syntax === "indented"), loadedUrls: used.loadedUrls };
  } catch (error) {
    throw Object.assign(new Error(error.message), {
      sassMessage: error.message,
      span: { start: { line: error.line - 1, column: error.column - 1 } },
    });
  }
};
exports.info = "dart-sass\t1.80.0";
exports.compileString = run;
exports.compileStringAsync = async (source, options) => run(source, options);
exports.initAsyncCompiler = async () => ({
  compileStringAsync: async (source, options) => run(source, options),
  dispose() {},
});
