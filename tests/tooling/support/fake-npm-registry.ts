import { writeFakeCommand } from "./fake-command.ts";

export function writeFakeNpmRegistry(binDir: string): void {
  writeFakeCommand(
    binDir,
    "curl",
    [
      "const fs = require('node:fs');",
      "const url = process.argv.at(-1);",
      "const name = decodeURIComponent(new URL(url).pathname.split('/')[1]);",
      "const version = process.env.NPM_FAKE_VERSION;",
      "const statePath = process.env.NPM_FAKE_STATE_PATH;",
      "const state = statePath && fs.existsSync(statePath)",
      "  ? JSON.parse(fs.readFileSync(statePath, 'utf8'))",
      "  : {};",
      "const requestsPath = process.env.NPM_FAKE_URL_LOG || (statePath && statePath + '.requests.jsonl');",
      "if (requestsPath) {",
      "  fs.appendFileSync(requestsPath, `${url}\\n`);",
      "}",
      "const exactVersion = url.includes(`/${version}?`);",
      // Concurrent version/tag reads must not write back over publisher state.
      "const versionChecks = exactVersion && requestsPath",
      "  ? fs.readFileSync(requestsPath, 'utf8').split('\\n').filter(request => request.split('?')[0] === url.split('?')[0]).length",
      "  : 0;",
      // The staged fixture exposes its tag before the exact version propagates.
      "const visible = process.env.NPM_FAKE_ALWAYS_VISIBLE === '1'",
      "  || state.published === true || state.published?.[name] === version",
      "  || state.versionVisible",
      "  || (state.publishCalls === 1 && (!exactVersion || versionChecks >= 3));",
      "if (!visible) {",
      "  process.stdout.write('{}VIZE_HTTP_STATUS:404');",
      "  process.exit(0);",
      "}",
      "const body = exactVersion",
      "  ? { name, version }",
      "  : { name, 'dist-tags': { latest: version, beta: version, rc: version } };",
      "process.stdout.write(`${JSON.stringify(body)}VIZE_HTTP_STATUS:200`);",
    ].join("\n"),
  );
}
