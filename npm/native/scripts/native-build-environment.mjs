import { randomBytes } from "node:crypto";

/** Every actual N-API build invocation receives a fresh identity, even with a warm Cargo cache. */
export function nativeBuildEnvironment(environment = process.env) {
  return { ...environment, VIZE_NATIVE_BUILD_TOKEN: randomBytes(16).toString("hex") };
}
