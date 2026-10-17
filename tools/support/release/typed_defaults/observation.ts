/** Shared custody and result fields retained by the public CLI observers. */
export interface SerializedFailure {
  name: string;
  message: string;
}

export interface BinaryMember {
  binary: string;
  bytes: number;
  mode: number;
  sha256: string;
  [field: string]: unknown;
}

export interface Distribution {
  member: BinaryMember;
  [field: string]: unknown;
}

export interface InputObservation {
  text: string;
  bytes: number;
  sha256: string;
  afterText: string | null;
  afterBytes: number | null;
  afterBase64: string | null;
  afterSha256: string | null;
  readError: SerializedFailure | null;
}

export interface Observation {
  index: number;
  suite: string;
  id: string;
  packet: string;
  sha256: string;
  passed: boolean;
}

export interface PacketResult {
  actual?: unknown;
  passed?: boolean;
  failure?: SerializedFailure;
}

export function serializeFailure(error: unknown): SerializedFailure {
  if (
    error !== null &&
    typeof error === "object" &&
    "name" in error &&
    typeof error.name === "string" &&
    "message" in error &&
    typeof error.message === "string"
  ) {
    return { name: error.name, message: error.message };
  }
  return { name: "Error", message: String(error) };
}
