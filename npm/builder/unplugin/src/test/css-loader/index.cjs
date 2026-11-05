// Test-only stand-in for css-loader with `exportType: "string"`. It sits in a
// `css-loader` directory so webpack-style-loader scopes the CSS before it, as
// it does for the real css-loader.
"use strict";

module.exports = function cssLoader(source) {
  return `export default ${JSON.stringify(source)};`;
};
