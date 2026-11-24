const fs = require("node:fs");
const args = process.argv.slice(2);
if (process.env.CURL_LOG) fs.appendFileSync(process.env.CURL_LOG, args.join(" ") + "\n");
const endpoint = args.at(-1).split("/");
const crateName = endpoint.at(-2);
const version = endpoint.at(-1);
if (crateName === process.env.TEST_CURL_FAIL_CRATE) {
  console.error("registry unavailable");
  process.exit(7);
}
if (crateName === process.env.TEST_CURL_MALFORMED_CRATE) {
  fs.writeSync(1, "{bad jsonVIZE_HTTP_STATUS:200");
  process.exit(0);
}
if (crateName === process.env.TEST_CURL_SCHEMA_CRATE) {
  fs.writeSync(1, JSON.stringify({ version: { crate: crateName } }) + "VIZE_HTTP_STATUS:200");
  process.exit(0);
}
if (crateName === process.env.TEST_CURL_SERVER_ERROR_CRATE) {
  fs.writeSync(1, JSON.stringify({ errors: [{ detail: "unavailable" }] }) + "VIZE_HTTP_STATUS:500");
  process.exit(22);
}
const published = (process.env.TEST_PUBLISHED_CRATES || "").split(",").includes(crateName);
const body = published
  ? { version: { crate: crateName, num: version } }
  : { errors: [{ detail: "Not Found" }] };
fs.writeSync(1, JSON.stringify(body) + "VIZE_HTTP_STATUS:" + (published ? "200" : "404"));
process.exit(published ? 0 : 22);
