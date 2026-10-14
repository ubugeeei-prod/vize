// Webpack/Rspack loader for the stand-in preprocessors in
// test-preprocessors.cjs. Sass and Stylus blocks use indentation syntax.
"use strict";

const { compile } = require("./test-preprocessors.cjs");

module.exports = function testPreprocessorLoader(source) {
  const query = new URLSearchParams((this.resourceQuery || "").slice(1));
  const lang = query.get("lang") ?? this.resourcePath.split(".").pop();
  return compile(source, lang === "sass" || lang === "stylus" || lang === "styl");
};
