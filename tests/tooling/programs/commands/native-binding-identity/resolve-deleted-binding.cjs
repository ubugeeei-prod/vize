const previous = localRequire(binary);
fs.unlinkSync(binary);
let calls = 0;
const cachedRequire = (specifier) => {
  calls++;
  return localRequire(specifier);
};
cachedRequire.resolve = () => {
  throw Error("original path unavailable");
};
assert.equal(requireWithBindingIdentity(cachedRequire, binary), previous);
assert.equal(calls, 1, "require must execute exactly once before optional resolution");
assert.equal(getBindingIdentity(previous).kind, "process");
const failure = Error("original load failure");
const failingRequire = () => {
  throw failure;
};
failingRequire.resolve = () => {
  throw Error("resolver must not run after failed loading");
};
assert.throws(
  () => requireWithBindingIdentity(failingRequire, binary),
  (error) => error === failure,
);
