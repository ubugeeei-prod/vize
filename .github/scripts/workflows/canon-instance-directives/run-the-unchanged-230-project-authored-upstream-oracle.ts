import fs from "node:fs";
console.log(
  JSON.parse(
    fs.readFileSync(
      process.env.RUNNER_TEMP + "/instance-directives/packets/providers.json",
      "utf8",
    ),
  ).binaries.native.path,
);
