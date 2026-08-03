import type { JsonRpcId } from "./protocol.ts";

export class LspRequestError extends Error {
  readonly code: number;
  readonly data: unknown;
  readonly id: JsonRpcId;
  readonly method: string;

  constructor(
    id: JsonRpcId,
    method: string,
    error: { code: number; message: string; data?: unknown },
  ) {
    super(`${method}: ${error.message}`);
    this.name = "LspRequestError";
    this.code = error.code;
    this.data = error.data;
    this.id = id;
    this.method = method;
  }
}
