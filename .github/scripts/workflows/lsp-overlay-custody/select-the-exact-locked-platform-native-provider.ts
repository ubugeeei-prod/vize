import assert from "node:assert/strict";
for (const name of [
  "CORSA_PATH",
  "CORSA_EXECUTABLE",
  "TSGO_PATH",
  "TSGO_EXECUTABLE",
  "TSRS_PATH",
  "TSRS_EXECUTABLE",
])
  assert.ok(!process.env[name], `ambient ${name} refused`);
