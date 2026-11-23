import { createRequire } from "node:module";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
try {
  await import(pathToFileURL(process.cwd() + "/npm/builder/shared/mapped-module.ts").href);
  const require = createRequire(pathToFileURL(process.cwd() + "/npm/builder/shared/package.json"));
  const filename = require.resolve("webpack-sources/package.json");
  console.log(
    JSON.stringify({ filename, version: JSON.parse(readFileSync(filename, "utf8")).version }),
  );
} catch (cause) {
  const error = cause as NodeJS.ErrnoException;
  console.error(error.stack);
  console.log(JSON.stringify({ code: error.code, message: error.message }));
  process.exitCode = 1;
}
