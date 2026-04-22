type VitePlusConfig = {
  pack?: unknown;
};

export function getTsdownPackConfig(config: VitePlusConfig) {
  if (config.pack == null) {
    throw new Error("Expected pack configuration in vite.config.ts");
  }

  return config.pack;
}
