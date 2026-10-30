/** Complete packet identities for the reported QSplitter project. */
import assert from "node:assert/strict";
import path from "node:path";
import { identify, hash } from "./canon-instance-directives-io.ts";
import { canonicalMappedGenerated, type Row } from "./canon-slot-constructor-audit.ts";

export function lines(source: string): number {
  return source.split(/\r\n|[\r\n\u2028\u2029]/u).length;
}

export function screenshotKey(row: Pick<Row, "file" | "code" | "message">): string {
  return `${row.file}\u0000${row.code}\u0000${row.message}`;
}

export function occurrences(source: string, needle: string): number {
  return source.split(needle).length - 1;
}

export function virtualText(result: Record<string, unknown>): string {
  assert.ok(typeof result.text === "string");
  return result.text;
}

export function authoredSpan(source: string) {
  const needle = "<QSplitter",
    opening = source.indexOf(needle);
  assert.ok(opening >= 0 && source.indexOf(needle, opening + 1) < 0);
  const start = opening + 1,
    end = start + "QSplitter".length,
    prefix = source.slice(0, start),
    previousLine = prefix.lastIndexOf("\n");
  return {
    text: source.slice(start, end),
    start,
    end,
    length: end - start,
    line: occurrences(prefix, "\n") + 1,
    column: start - previousLine,
    utf8Start: Buffer.byteLength(source.slice(0, start)),
    utf8End: Buffer.byteLength(source.slice(0, end)),
    coordinateSystem: "zero-based UTF-16 offsets; one-based line and UTF-16 column",
  };
}

function mappings(result: Record<string, unknown>): number[][] {
  assert.ok(Array.isArray(result.mappings));
  return result.mappings.map((value: unknown) => {
    assert.ok(
      Array.isArray(value) &&
        value.length === 6 &&
        value.every((part: unknown) => typeof part === "number" && Number.isSafeInteger(part)),
    );
    return value as number[];
  });
}

export function authoredMappings(
  result: Record<string, unknown>,
  span: ReturnType<typeof authoredSpan>,
) {
  const source = virtualText(result),
    matches = mappings(result)
      .filter(([, , original, originalLength]) => {
        assert.ok(original !== undefined && originalLength !== undefined);
        return original < span.utf8End && original + originalLength > span.utf8Start;
      })
      .map(([generated, length, original, originalLength, kind, features]) => {
        assert.ok(
          generated !== undefined &&
            length !== undefined &&
            original !== undefined &&
            originalLength !== undefined &&
            kind !== undefined &&
            features !== undefined,
        );
        return {
          generated: canonicalMappedGenerated(
            Buffer.from(source)
              .subarray(generated, generated + length)
              .toString("utf8"),
          ),
          original,
          originalLength,
          kind,
          features,
        };
      });
  assert.ok(matches.length > 0, "the exact authored component name must remain mapped");
  return matches;
}

function diagnosticPoint(source: string, line: number, column: number) {
  assert.ok(line >= 1 && column >= 1);
  let lineStart = 0,
    currentLine = 1;
  const newline = /\r\n|[\r\n\u2028\u2029]/gu;
  while (currentLine < line) {
    const match = newline.exec(source);
    assert.ok(match, `virtual line ${line} is outside the mapper packet`);
    lineStart = match.index + match[0].length;
    currentLine++;
  }
  newline.lastIndex = lineStart;
  const next = newline.exec(source),
    lineEnd = next?.index ?? source.length,
    offset = lineStart + column - 1;
  assert.ok(offset <= lineEnd, `virtual column ${column} is outside line ${line}`);
  return {
    utf16Offset: offset,
    utf8Offset: Buffer.byteLength(source.slice(0, offset)),
    lineText: source.slice(lineStart, lineEnd),
  };
}

export function diagnosticFingerprint(
  row: Row,
  result: Record<string, unknown>,
  sourceLineCount: number,
) {
  const source = virtualText(result),
    point = diagnosticPoint(source, row.line, row.column),
    mappedSegments = mappings(result).filter(([generated, length]) => {
      assert.ok(generated !== undefined && length !== undefined);
      return generated <= point.utf8Offset && point.utf8Offset < generated + length;
    }),
    virtualOnly = row.line > sourceLineCount && mappedSegments.length === 0,
    identity = {
      file: row.file,
      line: row.line,
      column: row.column,
      code: row.code,
      message: row.message,
      virtualTextSha256: hash(source),
      virtualUtf16Offset: point.utf16Offset,
      virtualUtf8Offset: point.utf8Offset,
      virtualLineSha256: hash(point.lineText),
      mappedAuthoredSegments: mappedSegments,
    };
  assert.ok(virtualOnly, `${row.file}:${row.line}:${row.column} must be explicitly virtual-only`);
  return { ...identity, fingerprintSha256: hash(JSON.stringify(identity)), virtualOnly };
}

export function packetFiles(root: string, packet: { result: Record<string, unknown> }) {
  return {
    receipt: identify(path.join(root, "receipt.json")),
    stdin: identify(path.join(root, "stdin.raw")),
    stdout: identify(path.join(root, "stdout.raw")),
    stderr: identify(path.join(root, "stderr.raw")),
    responses: identify(path.join(root, "responses.json")),
    virtualTextSha256: hash(virtualText(packet.result)),
  };
}
