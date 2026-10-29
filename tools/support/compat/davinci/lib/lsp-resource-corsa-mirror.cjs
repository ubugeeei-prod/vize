"use strict";

const targetAccess = "peer.value?.publicState";

function lineStartOffset(text, line) {
  if (!Number.isSafeInteger(line) || line < 0) throw new Error("invalid LSP line");
  let current = 0;
  let offset = 0;
  while (current < line) {
    const lf = text.indexOf("\n", offset);
    const cr = text.indexOf("\r", offset);
    const boundary = lf < 0 ? cr : cr < 0 ? lf : Math.min(lf, cr);
    if (boundary < 0) throw new Error("LSP line is outside mirrored document");
    offset = boundary + (text[boundary] === "\r" && text[boundary + 1] === "\n" ? 2 : 1);
    current++;
  }
  return offset;
}

function positionOffset(text, position) {
  if (!position || !Number.isSafeInteger(position.character) || position.character < 0) {
    throw new Error("invalid LSP character");
  }
  const start = lineStartOffset(text, position.line);
  const lf = text.indexOf("\n", start);
  const cr = text.indexOf("\r", start);
  const end = lf < 0 ? (cr < 0 ? text.length : cr) : cr < 0 ? lf : Math.min(lf, cr);
  if (start + position.character > end) throw new Error("LSP character is outside mirrored line");
  return start + position.character;
}

function nextMirroredDocument(current, message) {
  const params = message.params;
  if (message.method === "textDocument/didOpen") {
    const document = params && params.textDocument;
    if (!document || typeof document.uri !== "string" || typeof document.text !== "string") {
      throw new Error("invalid mirrored didOpen");
    }
    const first = document.text.indexOf(targetAccess);
    if (first < 0) return current;
    if (document.text.indexOf(targetAccess, first + 1) >= 0) {
      throw new Error("canonical hover target must be unique");
    }
    if (current && current.openedUri !== document.uri) {
      throw new Error("refusing a second canonical hover target");
    }
    return {
      openedUri: document.uri,
      text: document.text,
      version: document.version ?? null,
    };
  }
  if (message.method === "textDocument/didChange") {
    const document = params && params.textDocument;
    const changes = params && params.contentChanges;
    if (!document || !current || document.uri !== current.openedUri) return current;
    if (!Array.isArray(changes)) throw new Error("invalid mirrored didChange");
    let text = current.text;
    for (const change of changes) {
      if (!change || typeof change.text !== "string") throw new Error("invalid mirrored change");
      if (!change.range) {
        text = change.text;
        continue;
      }
      const start = positionOffset(text, change.range.start);
      const end = positionOffset(text, change.range.end);
      if (end < start) throw new Error("reversed mirrored change range");
      text = text.slice(0, start) + change.text + text.slice(end);
    }
    return {
      openedUri: current.openedUri,
      text,
      version: document.version ?? current.version,
    };
  }
  if (message.method === "textDocument/didClose") {
    const uri = params && params.textDocument && params.textDocument.uri;
    if (typeof uri === "string" && uri === current?.openedUri) return undefined;
  }
  return current;
}

module.exports = { nextMirroredDocument };
