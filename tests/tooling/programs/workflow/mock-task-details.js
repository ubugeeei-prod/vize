import fs from "node:fs";

const args = process.argv.slice(2);
fs.appendFileSync("calls.jsonl", JSON.stringify(args) + "\n");
if (args.includes("--last-details")) {
  process.stdout.write("task child exit=17\n");
  process.stderr.write("task stderr\n");
  const status = JSON.parse(process.env.VIZE_DETAILS_STATUS);
  if (typeof status === "string") process.kill(process.pid, status);
  else process.exit(status);
} else process.exit(Number(process.env.VIZE_ORIGINAL_STATUS));
