const fs = require("node:fs");

const binding = require(process.argv[2]);
const images = Object.entries(require.cache).filter(
  ([file, module]) => file.endsWith(".node") && module.exports === binding,
);
if (images.length !== 1) throw new Error("exact native image unavailable");
console.log(
  JSON.stringify({ image: fs.realpathSync(images[0][0]), token: binding.getNativeBuildToken() }),
);
