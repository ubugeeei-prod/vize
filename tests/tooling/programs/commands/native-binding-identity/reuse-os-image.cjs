const images = new Map();
require.extensions[".node"] = (module, filename) => {
  if (!images.has(filename)) images.set(filename, fs.readFileSync(filename, "utf8"));
  const body = images.get(filename);
  const token = crypto.createHash("sha256").update(body).digest("hex").slice(0, 32);
  module.exports = Object.freeze({ body, getNativeBuildToken: () => token });
};
const before = requireWithBindingIdentity(localRequire, binary);
const first = getBindingIdentity(before);
const helperCopy = path.join(__dirname, "independent-helper.cjs");
fs.copyFileSync(helper, helperCopy);
const other = localRequire(helperCopy);
assert.equal(
  other.getBindingIdentity(before),
  first,
  "same-isolate helper copies retain actual binding identity",
);
fs.writeFileSync(binary, newBody);
delete require.cache[binary];
const reloaded = other.requireWithBindingIdentity(localRequire, binary);
assert.notEqual(reloaded, before);
assert.equal(reloaded.body, oldBody, "model the independently observed same-path OS image reuse");
assert.equal(other.getBindingIdentity(reloaded).token, first.token);
assert.notEqual(
  other.getBindingIdentity(reloaded).token,
  crypto.createHash("sha256").update(newBody).digest("hex").slice(0, 32),
);
