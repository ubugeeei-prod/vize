import fs from "node:fs";
import crypto from "node:crypto";
import { createRequire } from "node:module";

const require = createRequire(process.env.IDENTITY_FIXTURE + "/worker.cjs");
require.extensions[".node"] = (module, filename) => {
  const body = fs.readFileSync(filename, "utf8");
  const token = crypto.createHash("sha256").update(body).digest("hex").slice(0, 32);
  module.exports =
    process.env.LEGACY_PROVIDER === "1" ? { body } : { body, getNativeBuildToken: () => token };
};
const { computePrecompileCacheKey: key } = await import(process.env.IDENTITY_FIXTURE + "/key.ts");
const first = key({ ssr: false, vapor: true });
const binding = require("@vizejs/native");
const reordered = key({ vapor: true, ssr: false });
const differentOptions = key({ vapor: false, ssr: false });
if (process.env.REPLACE_BODY === "1") {
  const binary = require.resolve("@vizejs/native/provider.node");
  const before = fs.statSync(binary);
  fs.writeFileSync(binary, "new-provider");
  fs.utimesSync(binary, before.atime, before.mtime);
}
const again = key({ ssr: false, vapor: true });
console.log(JSON.stringify({ first, reordered, differentOptions, again, body: binding.body }));
