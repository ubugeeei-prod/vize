const { compile } = require(require("./stand-in-path.json"));
exports.FileManager = class FileManager {};
exports.render = async (source) => {
  try {
    return { css: compile(source), imports: [] };
  } catch (error) {
    throw Object.assign(new Error(error.message), { line: error.line, column: error.column - 1 });
  }
};
