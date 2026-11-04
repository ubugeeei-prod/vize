import { normalHostArguments } from "./template-alias-paired-11822-hosts.ts";
import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { capture, prepareWorkspace, write } from "./template-alias-paired-11822-custody.ts";

export function semanticFiles() {
  const text = (invalid: boolean, directive: boolean) =>
    `<script setup lang="ts">\nconst value = { n: 1 };\n</script>\n<template>\n${directive ? "  <!-- @vue-expect-error -->\n" : ""}  {{ value.${invalid ? "missing" : "n"} }}\n</template>\n`;
  return new Map([
    ["Positive.vue", text(false, false)],
    ["Negative.vue", text(true, false)],
    ["Suppressed.vue", text(true, true)],
    ["Unused.vue", text(false, true)],
  ]);
}

export function semanticPacket(
  status: number | null,
  stdout: string,
  stderr: string,
  workspace: string,
) {
  assert.equal(status, 2);
  assert.equal(stderr, "");
  const lines = stdout
    .replaceAll(workspace + "/", "")
    .replaceAll("\r\n", "\n")
    .split("\n")
    .filter(Boolean);
  const packet = lines.map((line) => {
    const match =
      /^src\/(Negative|Unused)\.vue\(([0-9]+),([0-9]+)\): error TS([0-9]+): (.+)$/u.exec(line);
    assert.ok(match, "unexpected or unparsed semantic diagnostic: " + line);
    return {
      file: match[1],
      line: Number(match[2]),
      column: Number(match[3]),
      code: Number(match[4]),
      message: match[5],
    };
  });
  assert.equal(packet.length, 2, "complete independent template/directive oracle required");
  const negative = packet.filter((row) => row.file === "Negative"),
    unused = packet.filter((row) => row.file === "Unused");
  assert.equal(negative.length, 1);
  assert.equal(unused.length, 1);
  assert.equal(negative[0].code, 2339);
  assert.equal(negative[0].line, 5);
  assert.ok(negative[0].column >= 6 && negative[0].column <= 19);
  assert.ok(negative[0].message.includes("missing"));
  assert.equal(unused[0].code, 4);
  assert.equal(unused[0].line, 5);
  assert.ok(unused[0].column >= 8 && unused[0].column <= 24);
  assert.equal(unused[0].message, "Unused '@vue-expect-error' directive");
  return { status, stdout: lines.join("\n"), stderr, packet };
}

export async function qualifySemantics(options: {
  corpus: string;
  provider: string;
  output: string;
  packages: Record<string, string>;
  hosts: { name: string; command: string }[];
}) {
  const corpus = join(options.output, "semantic-inputs");
  mkdirSync(corpus);
  mkdirSync(join(corpus, "src"));
  for (const name of ["package.json", "tsconfig.base.json", "tsconfig.vize-content-mapper.json"])
    copyFileSync(join(options.corpus, name), join(corpus, name));
  for (const [name, source] of semanticFiles())
    writeFileSync(join(corpus, "src", name), source, { flag: "wx" });
  writeFileSync(
    join(corpus, "src/index.ts"),
    [...semanticFiles().keys()].map((name) => `import './${name}';`).join("\n") + "\n",
    { flag: "wx" },
  );
  const packets = new Map<string, ReturnType<typeof semanticPacket>>();
  for (const host of options.hosts)
    for (const id of ["currentmain", "candidate"] as const) {
      const directory = join(options.output, `semantic-${host.name}-${id}`),
        workspace = join(directory, "workspace");
      mkdirSync(directory);
      const closure = prepareWorkspace(
        corpus,
        options.provider,
        workspace,
        { id, package: options.packages[id] },
        undefined,
        4,
      );
      const row = await capture(
        join(directory, "observation"),
        process.execPath,
        normalHostArguments(host.command, closure.configFile),
        workspace,
      );
      const packet = semanticPacket(
        row.status,
        readFileSync(row.stdout.path, "utf8"),
        readFileSync(row.stderr.path, "utf8"),
        workspace,
      );
      if (packets.has(host.name)) assert.deepEqual(packet, packets.get(host.name));
      else packets.set(host.name, packet);
      write(join(directory, "qualified.json"), { timingEligible: false, packet });
    }
}
