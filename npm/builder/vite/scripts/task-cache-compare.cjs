const assert = require("node:assert/strict");
const { before, after } = JSON.parse(process.argv[2]);
assert.deepEqual(Object.keys(after).sort(), Object.keys(before).sort());
let checked = 0;
for (const name of Object.keys(after)) {
  const current = after[name],
    previous = before[name];
  if (current && typeof current === "object" && current.cache) {
    const now = typeof current.cache === "object" ? current.cache : current;
    const old = typeof previous.cache === "object" ? previous.cache : previous;
    const original = old.env ?? [];
    assert(Array.isArray(now.env));
    assert(now.env.includes("XDG_CACHE_HOME"));
    for (const key of original) assert(now.env.includes(key));
    assert.equal(new Set(now.env).size, now.env.length);
    assert.equal(now.env.length, new Set([...original, "XDG_CACHE_HOME"]).size);
    delete now.env;
    delete old.env;
    checked++;
  }
}
assert.equal(checked, 4);
assert.deepEqual(after, before, "all other normalized task fields are unchanged");
process.stdout.write(
  JSON.stringify({
    resolvedFactory: true,
    trackedEnvironmentVerified: checked,
    allOtherTaskFieldsUnchanged: true,
  }),
);
