const fs = require("node:fs");
const args = process.argv.slice(2);
if (
  process.env.VIZE_TEST_REQUIRE_TSGO !== "1" ||
  Object.hasOwn(process.env, "VIZE_TEST_DISABLE_TSGO")
)
  process.exit(92);
if (process.env.VIZE_NUXT_CONFIG_ITERATIONS !== "100") process.exit(93);
if (process.env.CORSA_PATH !== process.env.EXPECTED_CORSA_PATH) process.exit(94);
try {
  fs.accessSync(process.env.CORSA_PATH, fs.constants.X_OK);
} catch {
  process.exit(95);
}
if (args.includes("--archive-file")) process.exit(91);
fs.mkdirSync("target/nextest/" + process.env.NEXTEST_PROFILE, { recursive: true });
fs.writeFileSync(process.env.ARGV, JSON.stringify(args));
