// Only this runner-local receipt contains the worker's private argument vector.
import { readFileSync, writeFileSync } from "node:fs";
import { test } from "node:test";

void test("authenticate the installed isolated Node test worker", () => {
  const privatePath = new URL("./probe-private.json", import.meta.url);
  const argv = readFileSync("/proc/self/cmdline").toString("utf8").split("\0");
  if (argv.pop() !== "") throw new Error("Malformed private worker profile");
  writeFileSync(
    privatePath,
    JSON.stringify({
      argv,
      executable: process.execPath,
      nodeVersion: process.version,
      testWorker: process.env.NODE_TEST_CONTEXT === "child-v8",
    }),
    { flag: "wx", mode: 0o600 },
  );
});
