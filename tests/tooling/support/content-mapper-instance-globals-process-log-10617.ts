/** Exact process evidence for the private, immutable-after-capture 10617 producer. */
import assert from "node:assert/strict";
import fs from "node:fs";
import type { Packet } from "./content-mapper-instance-globals-10617.ts";

const closing = Buffer.from("\n]\n");

function writeAll(descriptor: number, body: Buffer, position: number) {
  let written = 0;
  while (written < body.length) {
    const size = fs.writeSync(descriptor, body, written, body.length - written, position + written);
    assert(Number.isSafeInteger(size) && size > 0 && size <= body.length - written);
    written += size;
  }
}

/**
 * This is not a log for arbitrary mutable objects: the owned caller constructs
 * plain Packets and never changes a captured Packet or its args afterward.
 * Keep the caller's references intact; no caller objects are cloned or frozen.
 * Each completed write equals JSON.stringify(allCapturedPackets, null, 2) + LF.
 */
export function createInstanceGlobalsProcessLog(file: string) {
  let position = 0;
  let identity: { dev: bigint; ino: bigint } | undefined;
  let failed = false;
  return (packet: Packet): void => {
    assert.equal(failed, false, "a failed process log cannot continue");
    try {
      // Serializing a singleton array keeps exactly the legacy array indentation
      // and escaping, without revisiting any earlier immutable process record.
      const pretty = JSON.stringify([packet], null, 2);
      const body = Buffer.from(`${identity ? ",\n" : "[\n"}${pretty.slice(2, -2)}\n]\n`);
      const descriptor = fs.openSync(file, identity ? "r+" : "w+");
      try {
        const stat = fs.fstatSync(descriptor, { bigint: true });
        assert(stat.isFile(), "process evidence must be a regular file");
        if (identity) {
          assert.deepEqual({ dev: stat.dev, ino: stat.ino }, identity, "process log replaced");
          assert.equal(stat.size, BigInt(position + closing.length), "process log size changed");
          const tail = Buffer.alloc(closing.length);
          assert.equal(fs.readSync(descriptor, tail, 0, tail.length, position), tail.length);
          assert(tail.equals(closing), "process log closing tail changed");
        }
        writeAll(descriptor, body, position);
        position += body.length - closing.length;
        identity = { dev: stat.dev, ino: stat.ino };
      } finally {
        fs.closeSync(descriptor);
      }
    } catch (error) {
      // Keep partial bytes visible. Never repair, retry or overwrite a failed
      // append with a later record that could conceal the original failure.
      failed = true;
      throw error;
    }
  };
}
