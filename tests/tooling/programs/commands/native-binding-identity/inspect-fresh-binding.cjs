require.extensions[".node"] = (m, f) => {
  const body = require("node:fs").readFileSync(f, "utf8");
  const token = require("node:crypto").createHash("sha256").update(body).digest("hex").slice(0, 32);
  m.exports = { body, getNativeBuildToken: () => token };
};
const h = require(process.argv[2]);
console.log(
  JSON.stringify(h.getBindingIdentity(h.requireWithBindingIdentity(require, process.argv[3]))),
);
