// Imported first by `run.ts`: stdout carries only the receipt, so the runtime's
// console output (including its development-build banner) is silenced before
// any Vue module evaluates.
export const stdoutLog = console.log;
console.log = () => {};
console.info = () => {};
