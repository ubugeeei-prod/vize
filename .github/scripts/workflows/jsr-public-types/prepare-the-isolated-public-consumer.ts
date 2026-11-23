import fs from "node:fs";
const root = process.env.RUNNER_TEMP + "/jsr-public-consumer";
fs.writeFileSync(root + "/package.json", JSON.stringify({ private: true, type: "module" }));
const config = JSON.parse(fs.readFileSync(root + "/deno.json", "utf8"));
const exact = "jsr:@vizejs/vize@" + process.env.PUBLIC_JSR_VERSION;
config.imports = Object.fromEntries(
  ["", "config", "native", "vite"].map((subpath) => {
    const suffix = subpath ? "/" + subpath : "";
    return ["@vizejs/vize" + suffix, exact + suffix];
  }),
);
fs.writeFileSync(root + "/deno.json", JSON.stringify(config, null, 2));
