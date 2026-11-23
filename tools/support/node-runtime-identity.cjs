const assert = require("node:assert/strict");

const facts = {
  version: process.version,
  platform: process.platform,
  arch: process.arch,
  execPath: process.execPath,
};
const fields = process.argv.slice(2);
assert.ok(fields.length && fields.every((field) => Object.hasOwn(facts, field)));
console.log(JSON.stringify(Object.fromEntries(fields.map((field) => [field, facts[field]]))));
