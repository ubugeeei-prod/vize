import * as fs from "fs";
import * as path from "path";
import * as zlib from "zlib";

export async function extractReleaseArchive(
  archivePath: string,
  destinationDir: string,
  exeName: string,
): Promise<void> {
  const archive = await fs.promises.readFile(archivePath);
  if (archivePath.endsWith(".zip")) {
    await extractServerFromZip(archive, destinationDir, exeName);
    return;
  }

  const tar = zlib.gunzipSync(archive);
  await extractServerFromTar(tar, destinationDir, exeName);
}

async function extractServerFromTar(
  tar: Buffer,
  destinationDir: string,
  exeName: string,
): Promise<void> {
  for (let offset = 0; offset + 512 <= tar.byteLength; offset += 512) {
    const header = tar.subarray(offset, offset + 512);
    if (header.every((byte) => byte === 0)) {
      break;
    }

    const name = readTarString(header, 0, 100);
    const prefix = readTarString(header, 345, 155);
    const entryName = prefix ? `${prefix}/${name}` : name;
    const size = parseInt(readTarString(header, 124, 12).trim() || "0", 8);
    const typeFlag = readTarString(header, 156, 1);
    const dataOffset = offset + 512;
    const nextOffset = dataOffset + Math.ceil(size / 512) * 512;

    if ((typeFlag === "" || typeFlag === "0") && path.basename(entryName) === exeName) {
      await writeExtractedServer(
        tar.subarray(dataOffset, dataOffset + size),
        path.join(destinationDir, exeName),
      );
      return;
    }

    offset = nextOffset - 512;
  }

  throw new Error(`archive did not contain ${exeName}`);
}

async function extractServerFromZip(
  zip: Buffer,
  destinationDir: string,
  exeName: string,
): Promise<void> {
  const eocdOffset = findEndOfCentralDirectory(zip);
  const entryCount = zip.readUInt16LE(eocdOffset + 10);
  let centralOffset = zip.readUInt32LE(eocdOffset + 16);

  for (let index = 0; index < entryCount; index++) {
    if (zip.readUInt32LE(centralOffset) !== 0x02014b50) {
      throw new Error("invalid zip central directory");
    }

    const compressionMethod = zip.readUInt16LE(centralOffset + 10);
    const compressedSize = zip.readUInt32LE(centralOffset + 20);
    const uncompressedSize = zip.readUInt32LE(centralOffset + 24);
    const fileNameLength = zip.readUInt16LE(centralOffset + 28);
    const extraLength = zip.readUInt16LE(centralOffset + 30);
    const commentLength = zip.readUInt16LE(centralOffset + 32);
    const localHeaderOffset = zip.readUInt32LE(centralOffset + 42);
    const entryName = zip
      .subarray(centralOffset + 46, centralOffset + 46 + fileNameLength)
      .toString("utf-8");

    if (path.basename(entryName) === exeName) {
      const extracted = extractZipEntry(zip, {
        compressedSize,
        compressionMethod,
        localHeaderOffset,
        uncompressedSize,
      });
      await writeExtractedServer(extracted, path.join(destinationDir, exeName));
      return;
    }

    centralOffset += 46 + fileNameLength + extraLength + commentLength;
  }

  throw new Error(`archive did not contain ${exeName}`);
}

function extractZipEntry(
  zip: Buffer,
  entry: {
    compressedSize: number;
    compressionMethod: number;
    localHeaderOffset: number;
    uncompressedSize: number;
  },
): Buffer {
  if (zip.readUInt32LE(entry.localHeaderOffset) !== 0x04034b50) {
    throw new Error("invalid zip local header");
  }

  const fileNameLength = zip.readUInt16LE(entry.localHeaderOffset + 26);
  const extraLength = zip.readUInt16LE(entry.localHeaderOffset + 28);
  const dataOffset = entry.localHeaderOffset + 30 + fileNameLength + extraLength;
  const compressed = zip.subarray(dataOffset, dataOffset + entry.compressedSize);

  if (entry.compressionMethod === 0) {
    if (compressed.byteLength !== entry.uncompressedSize) {
      throw new Error("stored zip entry has an unexpected size");
    }
    return compressed;
  }

  if (entry.compressionMethod === 8) {
    const inflated = zlib.inflateRawSync(compressed);
    if (inflated.byteLength !== entry.uncompressedSize) {
      throw new Error("deflated zip entry has an unexpected size");
    }
    return inflated;
  }

  throw new Error(`unsupported zip compression method ${entry.compressionMethod}`);
}

function findEndOfCentralDirectory(zip: Buffer): number {
  const minimumOffset = Math.max(0, zip.byteLength - 65_557);
  for (let offset = zip.byteLength - 22; offset >= minimumOffset; offset--) {
    if (zip.readUInt32LE(offset) === 0x06054b50) {
      return offset;
    }
  }

  throw new Error("invalid zip archive");
}

function readTarString(buffer: Buffer, start: number, length: number): string {
  const raw = buffer.subarray(start, start + length);
  const end = raw.indexOf(0);
  return raw.subarray(0, end === -1 ? raw.byteLength : end).toString("utf-8");
}

async function writeExtractedServer(contents: Buffer, serverPath: string): Promise<void> {
  await fs.promises.writeFile(serverPath, contents);
  if (process.platform !== "win32") {
    await fs.promises.chmod(serverPath, 0o755);
  }
}
