assert.throws(() => preloadProjectNative(custody), {
  name: "Error",
  message: "whole authored wrapper failure",
});
assert.equal(process.env.NAPI_RS_NATIVE_LIBRARY_PATH, undefined);
assert.equal(require.cache[custody.wrapper.path], undefined);
