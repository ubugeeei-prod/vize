/** Pass user runtime settings through to the shared native resolver. */
export function serverProcessEnv(env: NodeJS.ProcessEnv, debug: boolean): NodeJS.ProcessEnv {
  return debug ? { ...env, RUST_BACKTRACE: "1" } : env;
}
