import { resolve } from "node:path";
import { createRequire } from "node:module";
const require = createRequire(resolve("package.json"));
const native = require("@vizejs/native");
const identity = require("@vizejs/native/native-binding-identity.js").getBindingIdentity(native);
console.log(
  JSON.stringify({
    identity,
    token: native.getNativeBuildToken(),
    binary: require.resolve("@vizejs/native-linux-x64-gnu"),
  }),
);
