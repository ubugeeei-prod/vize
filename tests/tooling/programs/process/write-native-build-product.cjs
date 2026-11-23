const fs = require("node:fs");
const path = require("node:path");
module.exports = (library) => {
  fs.mkdirSync(path.dirname(library), { recursive: true });
  fs.writeFileSync(library, "fresh");
};
