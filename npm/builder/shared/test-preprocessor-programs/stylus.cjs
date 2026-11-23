const { compile } = require(require("./stand-in-path.json"));
module.exports = (source) => {
  const ref = {
    set: () => ref,
    define: () => ref,
    include: () => ref,
    import: () => ref,
    deps: () => [],
    render: (callback) => {
      let css;
      try {
        css = compile(source, !source.includes("{"));
      } catch (error) {
        throw Object.assign(new Error(error.message), { lineno: error.line, column: error.column });
      }
      return callback ? callback(null, css) : css;
    },
  };
  return ref;
};
