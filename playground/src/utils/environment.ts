export interface PlaygroundEnvironmentOptions {
  tab: string;
  wasmStatus: string;
}

export function getPlaygroundEnvironmentInfo(options: PlaygroundEnvironmentOptions): string {
  const { navigator, screen } = window;
  const userAgentData = "userAgentData" in navigator ? navigator.userAgentData : undefined;
  const platform =
    typeof userAgentData === "object" &&
    userAgentData !== null &&
    "platform" in userAgentData &&
    typeof userAgentData.platform === "string"
      ? userAgentData.platform
      : navigator.platform;

  return [
    `Vize: ${__VIZE_VERSION__}`,
    `Playground URL: ${window.location.href}`,
    `Active tab: ${options.tab}`,
    `WASM status: ${options.wasmStatus}`,
    `User agent: ${navigator.userAgent}`,
    `Platform: ${platform || "unknown"}`,
    `Language: ${navigator.language || "unknown"}`,
    `Viewport: ${window.innerWidth}x${window.innerHeight}`,
    `Screen: ${screen.width}x${screen.height}`,
    `Device pixel ratio: ${window.devicePixelRatio}`,
  ].join("\n");
}
