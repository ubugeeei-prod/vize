import { Transform } from "node:stream";

export default new Transform({
  writableObjectMode: true,
  transform(event, _encoding, callback) {
    if (event.type === "test:summary" && event.data.file) {
      callback(
        null,
        `${JSON.stringify({ file: event.data.file, duration_ms: event.data.duration_ms })}\n`,
      );
    } else {
      callback();
    }
  },
});
