import fs from "node:fs";
fs.writeFileSync(
  process.env.VIZE_NATIVE_ORIGINAL_FOR_CONSTANT_SFC_DOM_RAW_CAPTURE + ".statuses.json",
  JSON.stringify(
    {
      schema: "vize.native-sfc.original-for-constant-process-statuses",
      sourceRevision: process.env.GITHUB_SHA,
      run: process.env.GITHUB_RUN_ID,
      rustStatus: Number(process.env.RUST_STATUS),
      nodeStatus: Number(process.env.NODE_STATUS),
    },
    null,
    2,
  ) + "\n",
);
