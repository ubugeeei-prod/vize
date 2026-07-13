import { writeFakeCommand } from "./fake-command.ts";

export function installPublishedVersionCurl(binDir: string): void {
  writeFakeCommand(
    binDir,
    "curl",
    [
      "const endpoint = process.argv.at(-1).split('/');",
      "const crateName = endpoint.at(-2);",
      "const version = endpoint.at(-1);",
      "process.stdout.write(JSON.stringify({ version: { crate: crateName, num: version } }) + 'VIZE_HTTP_STATUS:200');",
      "process.exit(0);",
    ].join("\n"),
  );
}

export function installPublishCratesFakes(binDir: string): void {
  writeFakeCommand(
    binDir,
    "cargo",
    [
      "const fs = require('node:fs');",
      "const args = process.argv.slice(2);",
      "fs.appendFileSync(process.env.CARGO_LOG, args.join(' ') + '\\n');",
      "const [command] = args;",
      "if (command === 'package' && process.env.TEST_FAIL_PACKAGE) process.exit(1);",
      "const unresolved = (process.env.TEST_UNRESOLVED_CRATES || '').split(',');",
      "if (command === 'info' && unresolved.includes(args.at(-1).split('@')[0])) { console.error('not in registry index'); process.exit(1); }",
      "if (command === 'package' || command === 'publish' || command === 'info') process.exit(0);",
      "process.exit(1);",
    ].join("\n"),
  );
  writeFakeCommand(
    binDir,
    "curl",
    [
      "const fs = require('node:fs');",
      "const args = process.argv.slice(2);",
      "if (process.env.CURL_LOG) fs.appendFileSync(process.env.CURL_LOG, args.join(' ') + '\\n');",
      "const endpoint = args.at(-1).split('/');",
      "const crateName = endpoint.at(-2);",
      "const version = endpoint.at(-1);",
      "if (crateName === process.env.TEST_CURL_FAIL_CRATE) { console.error('registry unavailable'); process.exit(7); }",
      "if (crateName === process.env.TEST_CURL_MALFORMED_CRATE) { fs.writeSync(1, '{bad jsonVIZE_HTTP_STATUS:200'); process.exit(0); }",
      "if (crateName === process.env.TEST_CURL_SCHEMA_CRATE) { fs.writeSync(1, JSON.stringify({ version: { crate: crateName } }) + 'VIZE_HTTP_STATUS:200'); process.exit(0); }",
      "if (crateName === process.env.TEST_CURL_SERVER_ERROR_CRATE) { fs.writeSync(1, JSON.stringify({ errors: [{ detail: 'unavailable' }] }) + 'VIZE_HTTP_STATUS:500'); process.exit(22); }",
      "const published = (process.env.TEST_PUBLISHED_CRATES || '').split(',').includes(crateName);",
      "const body = published ? { version: { crate: crateName, num: version } } : { errors: [{ detail: 'Not Found' }] };",
      "fs.writeSync(1, JSON.stringify(body) + 'VIZE_HTTP_STATUS:' + (published ? '200' : '404'));",
      "process.exit(published ? 0 : 22);",
    ].join("\n"),
  );
}
