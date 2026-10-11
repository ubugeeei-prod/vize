import { createWriteStream } from "node:fs";
import { once } from "node:events";

// Chunking bounds serialization work between async writes, without truncating fields.
async function* tokens(value) {
  if (typeof value === "string") {
    yield '"';
    for (let offset = 0; offset < value.length; offset += 16_384) {
      yield JSON.stringify(value.slice(offset, offset + 16_384)).slice(1, -1);
    }
    yield '"';
  } else if (Array.isArray(value)) {
    yield "[";
    for (let index = 0; index < value.length; index++) {
      if (index) yield ",";
      yield* tokens(value[index]);
    }
    yield "]";
  } else if (value !== null && typeof value === "object") {
    yield "{";
    let separator = false;
    for (const [key, item] of Object.entries(value)) {
      if (item === undefined) continue;
      if (separator) yield ",";
      separator = true;
      yield* tokens(key);
      yield ":";
      yield* tokens(item);
    }
    yield "}";
  } else {
    yield JSON.stringify(value ?? null);
  }
}

export async function writeFailureJson(file, value, signal) {
  const stream = createWriteStream(file, { flags: "wx", signal });
  // Retain a rejection handler even when the caller's deadline wins its race.
  const finished = once(stream, "finish");
  void finished.catch(() => {});
  try {
    for await (const token of tokens(value)) {
      signal.throwIfAborted();
      if (!stream.write(token)) await once(stream, "drain");
    }
    stream.end("\n");
    await finished;
  } catch (error) {
    stream.destroy();
    throw error;
  }
}
