import type { Packet } from "./protocol-types.ts";

/** Full lossless framing; malformed packets are failures, never skipped. */
export class FrameDecoder {
  private buffer = Buffer.alloc(0);
  get remaining(): number {
    return this.buffer.length;
  }
  push(chunk: Buffer, receive: (packet: Packet) => void): void {
    this.buffer = Buffer.concat([this.buffer, chunk]);
    while (true) {
      const separator = this.buffer.indexOf("\r\n\r\n");
      if (separator < 0) return;
      const header = this.buffer.subarray(0, separator).toString("ascii");
      const lengths = header.split("\r\n").filter((line) => /^Content-Length:/i.test(line));
      if (lengths.length !== 1 || !/^Content-Length: *[0-9]+ *$/i.test(lengths[0]))
        throw new Error("malformed Content-Length header");
      const length = Number(lengths[0].split(":")[1].trim());
      if (!Number.isSafeInteger(length)) throw new Error("unsafe Content-Length");
      const end = separator + 4 + length;
      if (this.buffer.length < end) return;
      const packet: unknown = JSON.parse(
        new TextDecoder("utf8", { fatal: true }).decode(this.buffer.subarray(separator + 4, end)),
      );
      this.buffer = this.buffer.subarray(end);
      if (
        !packet ||
        typeof packet !== "object" ||
        Array.isArray(packet) ||
        (packet as Packet).jsonrpc !== "2.0"
      )
        throw new Error("invalid JSON-RPC packet");
      receive(packet as Packet);
    }
  }
}
