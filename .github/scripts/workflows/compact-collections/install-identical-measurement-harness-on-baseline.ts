import fs from "node:fs";
const path = "base/crates/vize_croquis_cf/Cargo.toml";
const source = fs.readFileSync(path, "utf8");
const dependency = "davinci_harness.workspace = true";
if (!source.includes(dependency))
  fs.writeFileSync(
    path,
    source.replaceAll("[dev-dependencies]", `[dev-dependencies]\n${dependency}`),
  );
