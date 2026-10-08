import { createHash } from "node:crypto";

type HashBytes = string | NodeJS.ArrayBufferView;
export const sha256 = (bytes: HashBytes): string =>
  createHash("sha256").update(bytes).digest("hex");
