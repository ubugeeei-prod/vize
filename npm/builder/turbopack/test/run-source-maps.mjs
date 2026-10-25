import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { AnyMap, originalPositionFor } from "@jridgewell/trace-mapping";
import { consumer, output, sha256 } from "./prepare-consumer.mjs";
import { readPublishedMap } from "./output-source-map.mjs";

/** Check final host maps, including the production scope-hoisting composition. */
export async function checkAuthoredMaps(sourcePath, original) {
  const maps = [];
  const authoredFilename = path.relative(consumer, sourcePath).replaceAll(path.sep, "/");
  const expectedLine = original.slice(0, original.indexOf("const emoji")).split("\n").length;
  await fs.mkdir(path.join(output, "maps"), { recursive: true });

  async function visit(directory, target) {
    for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
      const filename = path.join(directory, entry.name);
      if (entry.isDirectory()) await visit(filename, target);
      else if (entry.name.endsWith(".js") || entry.name.endsWith(".css")) {
        const published = await readPublishedMap(filename);
        if (!published) continue;
        const { bytes, bundle, reference, mapFile } = published;
        const traced = AnyMap(JSON.parse(bytes.toString("utf8")));
        const authored = new Set();
        const candidates = [];
        for (const [index, source] of traced.sources.entries()) {
          const decoded = decodeURI(source);
          if (!decoded.includes(authoredFilename)) continue;
          const content = traced.sourcesContent?.[index];
          candidates.push({
            source,
            sourceBytes: content == null ? null : Buffer.byteLength(content),
          });
          if (decoded.endsWith(authoredFilename) && traced.sourcesContent?.[index] === original)
            authored.add(traced.resolvedSources[index]);
        }
        if (!candidates.length) continue;
        const artifact = `next-${target}-${maps.length}`;
        await fs.writeFile(path.join(output, "maps", `${artifact}.map`), bytes);
        await fs.writeFile(
          path.join(output, "maps", `${artifact}${path.extname(filename)}`),
          bundle,
        );
        const packet = {
          file: path.relative(consumer, filename),
          mapFile: path.relative(consumer, mapFile),
          reference,
          target,
          sha256: sha256(bytes),
          generatedSha256: sha256(bundle),
          candidates,
          authored: [...authored],
          token: null,
        };
        maps.push(packet);
        if (!filename.endsWith(".js") || !authored.size) continue;
        for (const marker of bundle.matchAll(/🐱|\\u[dD]83[dD]\\u[dD][cC]31|\\u\{1[fF]431\}/g)) {
          const prefix = bundle.slice(0, marker.index);
          const generated = {
            line: prefix.split("\n").length,
            column: prefix.length - prefix.lastIndexOf("\n") - 1,
          };
          const origin = originalPositionFor(traced, generated);
          if (!authored.has(origin.source) || origin.line !== expectedLine) continue;
          packet.token = { generated, origin };
          break;
        }
      }
    }
  }
  await visit(path.join(consumer, ".next/static"), "client");
  await visit(path.join(consumer, ".next/server/chunks"), "server");
  await fs.writeFile(path.join(output, "next-source-maps.json"), JSON.stringify(maps, null, 2));
  for (const target of ["client", "server"])
    assert.ok(
      maps.some((map) => map.target === target && map.token),
      `${target} production JavaScript must map the surviving Unicode string to its original SFC declaration`,
    );
  assert.ok(
    maps.some((map) => map.file.endsWith(".css") && map.authored.length),
    "production CSS maps retain the complete original SFC",
  );
  return maps;
}
