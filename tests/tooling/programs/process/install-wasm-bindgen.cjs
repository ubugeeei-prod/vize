const fs = require("node:fs");
const path = require("node:path");
module.exports = (source) => {
  if (process.argv[2] !== "install") return;
  const destination = path.join(process.argv.at(-1), "bin");
  fs.mkdirSync(destination, { recursive: true });
  for (const name of ["wasm-bindgen", ".wasm-bindgen.cjs"]) {
    fs.copyFileSync(path.join(source, name), path.join(destination, name));
  }
};
