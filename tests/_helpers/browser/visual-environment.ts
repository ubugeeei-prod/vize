export function installVisualEnvironment(): void {
  const fixedNow = new Date("2026-01-01T00:00:00.000Z").valueOf();
  Object.defineProperty(Date, "now", { value: () => fixedNow });
  Object.defineProperty(Math, "random", { value: () => 0.42 });
  Object.defineProperty(navigator, "userAgent", {
    configurable: true,
    get: () => "Chromatic Playwright",
  });
}
