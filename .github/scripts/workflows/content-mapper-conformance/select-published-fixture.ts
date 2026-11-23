const fixtures = new Map([
  ["9499", "tests/_fixtures/content-mapper/with-defaults-surplus.json"],
  ["9498", "tests/_fixtures/content-mapper/escaton-public-props-9498.json"],
]);

const fixture = fixtures.get(process.argv[2] ?? "");
if (fixture === undefined) {
  console.error("Unsupported published report");
  process.exitCode = 2;
} else {
  console.log(fixture);
}
