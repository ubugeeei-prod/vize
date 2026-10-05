// Whole source-map segments and exact post-await UTF16 position ownership.
import assert from "node:assert/strict";

export function mappingFacts(code, map, source, name, SourceMapConsumer) {
  assert.equal(map.version, 3);
  assert.deepEqual(map.sourcesContent, [source]);
  assert.equal(map.sources.length, 1);
  const generatedLines = code.split("\n");
  const originalLines = source.split("\n");
  const consumer = new SourceMapConsumer(map);
  const mappings = [];
  consumer.eachMapping((entry) => {
    assert.ok(entry.generatedLine >= 1 && entry.generatedLine <= generatedLines.length);
    assert.ok(
      entry.generatedColumn >= 0 &&
        entry.generatedColumn <= generatedLines[entry.generatedLine - 1].length,
    );
    if (entry.originalLine !== null) {
      assert.ok(entry.originalLine >= 1 && entry.originalLine <= originalLines.length);
      assert.ok(
        entry.originalColumn >= 0 &&
          entry.originalColumn <= originalLines[entry.originalLine - 1].length,
      );
      assert.equal(entry.source, map.sources[0]);
    }
    mappings.push(entry);
  });
  assert.ok(mappings.length > 0);
  const token = name === "reported" ? "getCurrentInstance() !== null" : 'mark("end")';
  function position(text) {
    const index = text.indexOf(token);
    assert.ok(index >= 0 && text.indexOf(token, index + 1) < 0, token);
    const lines = text.slice(0, index).split("\n");
    return { line: lines.length, column: lines.at(-1).length };
  }
  const authored = position(source);
  const generated = position(code);
  const mapped = consumer.originalPositionFor(generated);
  assert.deepEqual(
    { line: mapped.line, column: mapped.column },
    authored,
    "the whole module maps the unchanged post-await observation to its exact authored UTF16 position",
  );
  return { mappings, postAwait: { token, authored, generated, mapped } };
}
