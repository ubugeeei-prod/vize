#!/usr/bin/env node
const { spawn, spawnSync } = require("node:child_process");
const { once } = require("node:events");

const decoder = "@@SYMBOLIZER_DECODER@@";
const args = process.argv.slice(2);
const marker = "0xdeadbeefbadc0ffe";
const delimiter = Buffer.from(",\n");

async function* lines(input) {
  let tail = Buffer.alloc(0);
  for await (const chunk of input) {
    const bytes = tail.length ? Buffer.concat([tail, chunk]) : chunk;
    let start = 0;
    for (let end = bytes.indexOf(10); end >= 0; end = bytes.indexOf(10, start)) {
      yield bytes.subarray(start, end + 1);
      start = end + 1;
    }
    tail = bytes.subarray(start);
  }
  if (tail.length) yield tail;
}

async function write(output, line) {
  if (!output.write(line)) await once(output, "drain");
}

async function forward(child, requests) {
  for await (const line of lines(process.stdin)) {
    const synthetic = line.equals(delimiter);
    const value = synthetic
      ? marker
      : line.toString("latin1").replace(/^[\t-\r ]+|[\t-\r ]+$/gu, "");
    if (!/^(?:0x)?[0-9a-fA-F]+$/u.test(value)) throw new Error("non-address decoder request");
    const address = BigInt("0x" + value.replace(/^0x/u, ""));
    requests.push({ address, synthetic });
    await write(child.stdin, Buffer.from(value + "\n"));
  }
  child.stdin.end();
}

async function decode(child, requests) {
  let current;
  let bodyLines = 0;
  let consumed = 0;
  function complete() {
    if (current && (bodyLines < 2 || bodyLines % 2))
      throw new Error("incomplete function/source frame");
  }
  for await (const line of lines(child.stdout)) {
    if (line.at(-1) !== 10) throw new Error("truncated decoder line");
    const header = line.toString("latin1").replace(/[\r\n]+$/u, "");
    if (/^0x[0-9a-fA-F]+$/u.test(header)) {
      complete();
      current = requests[consumed++];
      if (!current) throw new Error("unsolicited address header");
      if (BigInt(header) !== current.address)
        throw new Error("address header differs from ordered request");
      bodyLines = 0;
      // perf can issue its next request as soon as it sees its delimiter.
      await write(process.stdout, current.synthetic ? delimiter : line);
      if (consumed === requests.length) {
        requests.length = 0;
        consumed = 0;
      }
    } else {
      if (!current) throw new Error("body without an address header");
      bodyLines++;
      if (!current.synthetic) await write(process.stdout, line);
    }
  }
  complete();
  if (consumed !== requests.length) throw new Error("missing decoder response");
}

async function run() {
  if (args.includes("--version")) {
    const result = spawnSync(decoder, args, { stdio: "inherit" });
    if (result.error) throw result.error;
    return result.status ?? 1;
  }
  const child = spawn(decoder, args, { stdio: ["pipe", "pipe", "inherit"] });
  const requests = [];
  let timeout;
  try {
    const exit = once(child, "close").then(([code]) => code ?? 1);
    const input = forward(child, requests);
    const output = decode(child, requests);
    // Once the decoder has exited, bound the wait for a producer that left stdin open.
    const deadline = exit.then(
      () =>
        new Promise((_, reject) => {
          timeout = setTimeout(() => reject(new Error("unfinished decoder input")), 1000).unref();
        }),
    );
    const failure = once(child.stdin, "error").then(([error]) => {
      throw error;
    });
    const [code] = await Promise.race([Promise.all([exit, input, output]), deadline, failure]);
    return code;
  } catch (error) {
    child.kill("SIGKILL");
    process.stdin.destroy();
    throw error;
  } finally {
    clearTimeout(timeout);
  }
}

run().then(
  (status) => {
    process.exitCode = status;
  },
  (error) => {
    process.stderr.write("addr2line sentinel frame failed: " + error.message + "\n");
    process.exitCode = 1;
  },
);
