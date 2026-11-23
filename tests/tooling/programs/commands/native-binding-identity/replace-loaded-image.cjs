require.extensions[".node"] = (module, filename) => {
  const replacement = filename + ".replacement";
  const stat = fs.statSync(filename);
  fs.writeFileSync(replacement, newBody);
  fs.utimesSync(replacement, stat.atime, stat.mtime);
  fs.renameSync(replacement, filename);
  module.exports = { body: newBody, getNativeBuildToken: () => "11111111111111111111111111111111" };
};
const binding = requireWithBindingIdentity(localRequire, binary);
assert.equal(binding.body, newBody);
assert.deepEqual(getBindingIdentity(binding), {
  kind: "native",
  token: "11111111111111111111111111111111",
});
