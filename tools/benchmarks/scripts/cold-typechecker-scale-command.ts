/** Resolve an already prepared cold fixture into a direct measured command. */
import { createRequire } from "node:module";
import { dirname, join } from "node:path";

export function coldScaleCommand(options: {
  provider: string;
  out: string;
  app: string;
  tool: string;
  binary?: string;
}) {
  const { provider, out, tool } = options;
  const require = createRequire(join(provider, "package.json"));
  const nativePackage =
    process.platform === "darwin"
      ? `@typescript/typescript-darwin-${process.arch}`
      : `@typescript/typescript-linux-${process.arch}`;
  const nativeRequire = createRequire(require.resolve("typescript-native/package.json"));
  const checker = join(
    dirname(nativeRequire.resolve(nativePackage + "/package.json")),
    "lib",
    "tsc",
  );
  const cli = options.binary ?? join(provider, "node_modules", "vize", "bin", "vize");
  const executable =
    tool === "mapper"
      ? checker
      : tool === "vue-tsc"
        ? process.execPath
        : options.binary
          ? cli
          : process.execPath;
  const args =
    tool === "mapper"
      ? ["--runExternalCode", "--noEmit", "--pretty", "false", "-p", "tsconfig.json"]
      : tool === "vue-tsc"
        ? [
            join(provider, "node_modules", "vue-tsc", "bin", "vue-tsc.js"),
            "--noEmit",
            "--pretty",
            "false",
            "-p",
            "tsconfig.json",
          ]
        : [
            ...(options.binary ? [] : [cli]),
            "check",
            "--no-config",
            "--format",
            "json",
            "--tsconfig",
            "tsconfig.json",
            "--corsa-path",
            checker,
            ...(tool === "profile" ? ["--profile-json", join(out, "profile.json")] : []),
          ];
  return {
    backend: checker,
    command: {
      executable,
      args,
      cwd: options.app,
      env: { NO_COLOR: "1", XDG_CACHE_HOME: join(out, "xdg-cache"), TMPDIR: join(out, "tmp") },
    },
  };
}
