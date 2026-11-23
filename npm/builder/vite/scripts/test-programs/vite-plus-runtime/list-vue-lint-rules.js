import fs from "node:fs";
fs.appendFileSync(process.env.VIZE_RULE_CATALOG_TEST_COUNTER, "spawn\n");
process.stdout.write(
  JSON.stringify([
    { scope: "vue", value: "no-v-html" },
    { scope: "eslint", value: "eqeqeq" },
  ]),
);
