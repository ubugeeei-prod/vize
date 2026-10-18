import { fakeJsonStateTransaction, writeFakeCommand } from "./fake-command.ts";

// Existing metadata fixtures now model the archive required by successful publication.
export const fakeNpmArchiveSource = [
  "const fakeArchive = Buffer.from('fixture npm archive\\n');",
  "const fakeDist = (name, version) => ({",
  "  tarball: `https://registry.npmjs.org/${name}/-/fixture-${version}.tgz`,",
  "  integrity: 'sha512-' + require('node:crypto').createHash('sha512').update(fakeArchive).digest('base64'),",
  "});",
  "if (new URL(process.argv.at(-1)).pathname.endsWith('.tgz')) {",
  "  if (process.env.NPM_FAKE_URL_LOG) require('node:fs').appendFileSync(process.env.NPM_FAKE_URL_LOG, process.argv.at(-1) + '\\n');",
  "  process.stdout.write(fakeArchive);",
  "  process.stderr.write('VIZE_HTTP_STATUS:200');",
  "  process.exit(0);",
  "}",
].join("\n");

export function writeFakeNpmRegistry(binDir: string): void {
  writeFakeCommand(
    binDir,
    "curl",
    [
      fakeNpmArchiveSource,
      "const fs = require('node:fs');",
      "const url = process.argv.at(-1);",
      "const name = decodeURIComponent(new URL(url).pathname.split('/')[1]);",
      "const version = process.env.NPM_FAKE_VERSION;",
      "const statePath = process.env.NPM_FAKE_STATE_PATH;",
      fakeJsonStateTransaction,
      "if (process.env.NPM_FAKE_URL_LOG) {",
      "  fs.appendFileSync(process.env.NPM_FAKE_URL_LOG, `${url}\\n`);",
      "}",
      "const exactVersion = url.includes(`/${version}?`);",
      "const update = (state) => {",
      "  if (exactVersion) state.versionChecks = (state.versionChecks || 0) + 1;",
      "  else state.tagChecks = (state.tagChecks || 0) + 1;",
      "};",
      "const state = statePath && fs.existsSync(statePath)",
      "  ? mutateFakeJsonState(statePath, update) : {};",
      "if (!statePath || !fs.existsSync(statePath)) update(state);",
      "const visible = process.env.NPM_FAKE_ALWAYS_VISIBLE === '1'",
      "  || state.published === true || state.published?.[name] === version",
      "  || state.versionVisible",
      "  || (state.publishCalls === 1 && (!exactVersion || state.versionChecks >= 3));",
      "if (!visible) {",
      "  process.stdout.write('{}VIZE_HTTP_STATUS:404');",
      "  process.exit(0);",
      "}",
      "const body = exactVersion",
      "  ? { name, version, dist: fakeDist(name, version) }",
      "  : { name, 'dist-tags': { latest: version, beta: version, rc: version } };",
      "process.stdout.write(`${JSON.stringify(body)}VIZE_HTTP_STATUS:200`);",
    ].join("\n"),
  );
}
