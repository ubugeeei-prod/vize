const fs = require("node:fs");
module.exports = (image) => {
  fs.writeFileSync(image, "image:" + process.env.VIZE_NATIVE_BUILD_TOKEN);
};
