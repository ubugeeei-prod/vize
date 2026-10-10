import assert from "node:assert/strict";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { toolDefinitions } from "../../npm/mcp-musea/src/tools/definitions.ts";

const root = path.resolve(import.meta.dirname, "../..");
const documents = ["en", "ja"].map((locale) => ({
  locale,
  source: readFileSync(
    path.join(root, "docs/content", locale === "en" ? "" : locale, "integrations/mcp.md"),
    "utf8",
  ),
}));
const manifest = JSON.parse(readFileSync(path.join(root, "npm/mcp-musea/package.json"), "utf8"));

function servers(source: string) {
  return [...source.matchAll(/\x60\x60\x60json\n([\s\S]*?)\n\x60\x60\x60/g)].map((match) => {
    const config = JSON.parse(match[1].replace(/^\s*\/\/.*$/gm, ""));
    assert.deepEqual(Object.keys(config.mcpServers), ["vize-musea"]);
    return config.mcpServers["vize-musea"] as { command: string; args: string[]; type?: string };
  });
}

test("MCP client samples launch the installed binary from an explicit project root", () => {
  for (const { locale, source } of documents) {
    const entries = servers(source);
    assert.equal(entries.length, 2, locale + ": preserve Code and Desktop JSON examples");
    assert.match(source, /\.mcp\.json/);
    assert.doesNotMatch(source, /\.claude\/settings\.json/);
    assert.match(source, /claude mcp add --transport stdio --scope project vize-musea --/);
    assert.equal(entries[0].type, "stdio");
    assert.equal(entries[0].command, "vp");
    assert.ok(path.isAbsolute(entries[1].command), locale + ": Desktop needs an absolute vp path");
    for (const entry of entries) {
      const [directoryFlag, directory, action, binary, projectRoot] = entry.args;
      assert.equal(entry.args.length, 5);
      assert.equal(directoryFlag, "-C");
      assert.equal(action, "exec");
      assert.ok(manifest.bin[binary], locale + ": command must exist in the package manifest");
      assert.ok(path.isAbsolute(directory));
      assert.equal(projectRoot, directory);
    }
  }
  assert.deepEqual(servers(documents[0].source), servers(documents[1].source));
});

test("documented server arguments reach the actual CLI parser without relying on client cwd", () => {
  const temporary = mkdtempSync(path.join(os.tmpdir(), "vize-docs-mcp-"));
  try {
    const original = readFileSync(path.join(root, "npm/mcp-musea/src/cli.ts"), "utf8");
    assert.ok(original.includes('import { startServer } from "./index.js";'));
    writeFileSync(
      path.join(temporary, "cli.mjs"),
      original.replace('from "./index.js"', 'from "./index.mjs"'),
    );
    writeFileSync(
      path.join(temporary, "index.mjs"),
      "export async function startServer(projectRoot) { console.log(JSON.stringify({ projectRoot })); }\n",
    );
    const readme = readFileSync(path.join(root, "npm/mcp-musea/README.md"), "utf8");
    const block = readme.match(/\x60\x60\x60json\n([\s\S]*?)\n\x60\x60\x60/);
    assert.ok(block, "preserve the package README client example");
    const examples = [...servers(documents[0].source), JSON.parse(block[1]).mcpServers.musea];
    for (const entry of examples) {
      const result = spawnSync(
        process.execPath,
        [path.join(temporary, "cli.mjs"), ...entry.args.slice(4)],
        {
          cwd: temporary,
          env: { ...process.env, MUSEA_PROJECT_ROOT: "/different/client-default" },
          encoding: "utf8",
        },
      );
      assert.equal(result.status, 0, result.stderr);
      assert.deepEqual(JSON.parse(result.stdout), { projectRoot: entry.args[1] });
    }
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});

test("both MCP guides describe every currently advertised tool", () => {
  for (const { locale, source } of documents) {
    for (const { name } of toolDefinitions) {
      assert.ok(source.includes(name), locale + ": missing advertised tool " + name);
    }
  }
});
