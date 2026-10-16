import fs from "node:fs";
import path from "node:path";
import { isDeepStrictEqual } from "node:util";
import {
  errorName,
  runCapture,
  sha256File,
  statIdentity,
  writeJson,
} from "./native-fault-support.ts";
import type { Packet } from "./native-fault-support.ts";

function readAt(descriptor: number, size: number, offset: number): Buffer {
  if (!Number.isSafeInteger(offset) || offset < 0) throw new Error("Invalid ELF offset");
  const buffer = Buffer.alloc(size);
  if (fs.readSync(descriptor, buffer, 0, size, offset) !== size)
    throw new Error("Truncated ELF authority");
  return buffer;
}

function validElf(header: Buffer): boolean {
  return header.subarray(0, 6).equals(Buffer.from([0x7f, 0x45, 0x4c, 0x46, 2, 1]));
}

export function corePid(core: string): number {
  const descriptor = fs.openSync(core, "r");
  try {
    const header = readAt(descriptor, 64, 0);
    if (!validElf(header) || header.readUInt16LE(16) !== 4 || header.readUInt16LE(18) !== 62) {
      throw new Error("Expected Linux x86_64 ET_CORE");
    }
    const offset = Number(header.readBigUInt64LE(32));
    const size = header.readUInt16LE(54);
    const count = header.readUInt16LE(56);
    if (size !== 56 || count === 0 || count > 4096)
      throw new Error("Invalid core program-header authority");
    for (let index = 0; index < count; index += 1) {
      const program = readAt(descriptor, size, offset + index * size);
      if (program.readUInt32LE(0) !== 4) continue;
      const noteOffset = Number(program.readBigUInt64LE(8));
      const noteSize = Number(program.readBigUInt64LE(32));
      if (noteSize > 1_048_576) throw new Error("Core notes exceed bounded identity parser");
      const notes = readAt(descriptor, noteSize, noteOffset);
      for (let cursor = 0; cursor < notes.length;) {
        if (cursor + 12 > notes.length) throw new Error("Truncated core note header");
        const nameSize = notes.readUInt32LE(cursor);
        const descriptorSize = notes.readUInt32LE(cursor + 4);
        const kind = notes.readUInt32LE(cursor + 8);
        const nameStart = cursor + 12;
        const descriptorStart = nameStart + Math.ceil(nameSize / 4) * 4;
        const end = descriptorStart + descriptorSize;
        if (end > notes.length) throw new Error("Truncated core note descriptor");
        if (
          kind === 3 &&
          notes.subarray(nameStart, nameStart + nameSize).equals(Buffer.from("CORE\0"))
        ) {
          if (descriptorSize < 28) throw new Error("Truncated Linux PRPSINFO");
          return notes.readInt32LE(descriptorStart + 24);
        }
        cursor = descriptorStart + Math.ceil(descriptorSize / 4) * 4;
      }
    }
  } finally {
    fs.closeSync(descriptor);
  }
  throw new Error("Core has no authenticated Linux PRPSINFO process PID");
}

export function producer(core: string, output: string, hashFile = sha256File): Packet {
  const pid = corePid(core);
  if (path.basename(core) !== `core.${pid}`)
    throw new Error("Core filename does not match actual producer PID");
  const processDirectory = path.join(output, "processes");
  const packets = fs
    .readdirSync(processDirectory)
    .filter((name) => name.startsWith(`${pid}-`) && name.endsWith(".json"))
    .map(
      (name) => JSON.parse(fs.readFileSync(path.join(processDirectory, name), "utf8")) as Packet,
    );
  if (!packets.length || new Set(packets.map((packet) => packet.startTicks)).size !== 1) {
    throw new Error("Missing or ambiguous retained producer start ticks");
  }
  const identities = packets.flatMap((packet) =>
    packet.providers
      .filter((item: Packet) => item.path === packet.executable && item.sha256)
      .map((item: Packet) => ({ ...item, startTicks: packet.startTicks, pid })),
  );
  if (!identities.length || identities.some((item) => !isDeepStrictEqual(item, identities[0]))) {
    throw new Error("Missing or changed observed producer executable");
  }
  const identity = identities[0];
  const before = fs.statSync(identity.path, { bigint: true });
  const descriptor = fs.openSync(identity.path, "r");
  let header: Buffer;
  try {
    header = readAt(descriptor, 64, 0);
  } finally {
    fs.closeSync(descriptor);
  }
  if (
    !validElf(header) ||
    ![2, 3].includes(header.readUInt16LE(16)) ||
    header.readUInt16LE(18) !== 62
  ) {
    throw new Error("Producer is not an authenticated x86_64 executable ELF");
  }
  if (before.ino !== BigInt(identity.mappedInode) || before.size !== BigInt(identity.bytes)) {
    throw new Error("Physical producer inode or size changed");
  }
  if (hashFile(identity.path) !== identity.sha256)
    throw new Error("Physical producer executable hash changed");
  if (statIdentity(before) !== statIdentity(fs.statSync(identity.path, { bigint: true }))) {
    throw new Error("Producer executable changed during authentication");
  }
  return identity;
}

function hasCommand(command: string): boolean {
  return (process.env.PATH ?? "").split(path.delimiter).some((directory) => {
    try {
      const filename = path.join(directory, command);
      fs.accessSync(filename, fs.constants.X_OK);
      return fs.statSync(filename).isFile();
    } catch {
      return false;
    }
  });
}

export function diagnoseCore(core: string, receipt: Packet, output: string): void {
  const details: Packet = {
    file: path.basename(core),
    bytes: fs.statSync(core).size,
    sha256: sha256File(core),
    rawCore: "runner-local; not published",
  };
  let identity: Packet;
  try {
    identity = producer(core, output);
  } catch (error) {
    details.producerUnavailable = errorName(error);
    receipt.cores.push(details);
    return;
  }
  details.producer = identity;
  if (!hasCommand("gdb"))
    receipt.gdbInstall = runCapture([
      "sudo",
      "-n",
      "apt-get",
      "install",
      "-y",
      "--no-install-recommends",
      "gdb",
    ]);
  if (hasCommand("gdb")) {
    const result = runCapture(
      [
        "gdb",
        "--batch",
        "-nx",
        "-iex",
        "set auto-load off",
        "-iex",
        "set debuginfod enabled off",
        "-ex",
        "set pagination off",
        "-ex",
        "set print frame-arguments none",
        "-ex",
        "set print entry-values no",
        "-ex",
        "echo VIZE_NATIVE_FAULT_BEGIN\\n",
        "-ex",
        "info target",
        "-ex",
        "thread apply all bt",
        "-ex",
        "info sharedlibrary",
        "-ex",
        "info registers",
        "-ex",
        "echo VIZE_NATIVE_FAULT_END\\n",
        identity.path,
        core,
      ],
      true,
    );
    writeJson(path.join(output, `${path.basename(core)}-gdb.json`), result);
    details.gdbStatus = result.status;
  } else details.gdbUnavailable = true;
  receipt.cores.push(details);
}
