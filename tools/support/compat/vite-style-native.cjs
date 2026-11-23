const fs = require("fs"),
  crypto = require("crypto"),
  { createRequire } = require("module");
const req = createRequire(process.argv[2] + "/package.json");
const entry = req.resolve("@vizejs/native");
req("@vizejs/native");
const images = Object.keys(require.cache)
  .filter((file) => file.endsWith(".node"))
  .map((file) => ({
    file: fs.realpathSync(file),
    sha256: crypto.createHash("sha256").update(fs.readFileSync(file)).digest("hex"),
  }));
process.stdout.write(JSON.stringify({ entry: fs.realpathSync(entry), images }));
