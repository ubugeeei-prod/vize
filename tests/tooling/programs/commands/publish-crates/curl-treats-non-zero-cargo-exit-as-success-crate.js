require("node:fs").writeSync(
  1,
  JSON.stringify({ errors: [{ detail: "Not Found" }] }) + "VIZE_HTTP_STATUS:404",
);
process.exit(22);
