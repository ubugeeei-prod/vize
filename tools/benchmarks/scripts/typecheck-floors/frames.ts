/** Content-Length framing for the floor mappers: raw bodies in, framed bodies out. */

/** Incremental reader of framed message bodies, left as bytes. */
export function bodyReader(): (chunk: Buffer) => Buffer[] {
  let buffer: Buffer = Buffer.alloc(0);
  return (chunk) => {
    buffer = buffer.length > 0 ? Buffer.concat([buffer, chunk]) : chunk;
    const bodies: Buffer[] = [];
    for (;;) {
      const end = buffer.indexOf("\r\n\r\n");
      if (end < 0) return bodies;
      const header = buffer.subarray(0, end).toString("ascii");
      const length = /^Content-Length: *(\d+)$/imu.exec(header)?.[1];
      if (length == null) throw new Error(`frame header without Content-Length: ${header}`);
      const start = end + 4;
      if (buffer.length < start + Number(length)) return bodies;
      bodies.push(buffer.subarray(start, start + Number(length)));
      buffer = buffer.subarray(start + Number(length));
    }
  };
}

export function writeFrame(out: NodeJS.WritableStream, body: Buffer): void {
  out.write(`Content-Length: ${body.length}\r\n\r\n`);
  out.write(body);
}

export const writeMessage = (out: NodeJS.WritableStream, message: unknown): void =>
  writeFrame(out, Buffer.from(JSON.stringify(message)));
