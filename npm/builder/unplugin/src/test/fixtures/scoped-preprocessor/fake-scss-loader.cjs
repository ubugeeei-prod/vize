"use strict";

const { compile } = require("./fake-scss.cjs");

module.exports = function fakeScssLoader(source) {
  return compile(source);
};
