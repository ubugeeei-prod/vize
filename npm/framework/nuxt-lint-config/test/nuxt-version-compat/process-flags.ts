export function f() {
  if (process.client) {
    return window.innerWidth;
  }
  return process.server ? 0 : 1;
}
