const requireFromWorkspace = require("node:module").createRequire(
  require("node:path").resolve("package.json"),
);
requireFromWorkspace("@vscode/test-electron")
  .downloadAndUnzipVSCode({ version: process.env.VSCODE_VERSION, timeout: 30000 })
  .then((p) => require("fs").writeFileSync(process.env.HOME + "/code-path", p));
