require.extensions[".node"] = (module, filename) => {
  module.exports = Object.freeze({ body: fs.readFileSync(filename, "utf8") });
};
const previous = localRequire(binary);
fs.writeFileSync(binary, newBody);
assert.equal(requireWithBindingIdentity(localRequire, binary), previous);
const identity = getBindingIdentity(previous);
assert.equal(identity.kind, "process");
assert.equal(getBindingIdentity(previous), identity);
assert.equal(previous.body, oldBody);
const custom = path.join(__dirname, "custom.cjs");
fs.writeFileSync(
  custom,
  'module.exports = Object.freeze({ getNativeBuildToken: () => "00000000000000000000000000000000" });',
);
const customBinding = requireWithBindingIdentity(localRequire, custom);
const customIdentity = getBindingIdentity(customBinding);
assert.equal(customIdentity.kind, "process");
assert.notEqual(customIdentity.nonce, identity.nonce);
assert.equal(Object.isFrozen(customIdentity), true);
