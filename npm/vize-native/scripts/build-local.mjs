import { copyFileSync, existsSync, mkdirSync, readdirSync, rmSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync, spawnSync } from "node:child_process";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const packageDir = path.resolve(scriptDir, "..");
const outputDir = path.join(packageDir, ".artifacts", "native");
const isRelease = process.argv.includes("--release");

const resolveMacOsSdkRoot = () => {
  if (process.env.SDKROOT?.trim()) {
    return process.env.SDKROOT.trim();
  }

  for (const args of [["--sdk", "macosx", "--show-sdk-path"], ["--show-sdk-path"]]) {
    try {
      return execFileSync("xcrun", args, {
        encoding: "utf8",
        stdio: ["ignore", "pipe", "ignore"],
      }).trim();
    } catch {}
  }

  const fallbackSdkRoots = [
    "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk",
    "/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk",
  ];

  return fallbackSdkRoots.find((sdkRoot) => existsSync(sdkRoot));
};

const resolveDarwinBuildEnv = () => {
  if (process.platform !== "darwin") {
    return process.env;
  }

  const env = {
    ...process.env,
    CC: process.env.CC ?? "clang",
    CXX: process.env.CXX ?? "clang++",
  };

  const sdkRoot = resolveMacOsSdkRoot();

  if (!sdkRoot) {
    return env;
  }

  env.SDKROOT = sdkRoot;

  if (!env.LIBRARY_PATH?.split(":").includes(path.join(sdkRoot, "usr/lib"))) {
    env.LIBRARY_PATH = [path.join(sdkRoot, "usr/lib"), env.LIBRARY_PATH].filter(Boolean).join(":");
  }

  return env;
};

rmSync(outputDir, { recursive: true, force: true });
mkdirSync(outputDir, { recursive: true });

const buildArgs = [
  "exec",
  "napi",
  "build",
  "--platform",
  "--manifest-path",
  "../../crates/vize_vitrine/Cargo.toml",
  "-p",
  "vize_vitrine",
  "--features",
  "napi",
  "--output-dir",
  outputDir,
  "--no-js",
];

if (isRelease) {
  buildArgs.splice(4, 0, "--release");
}

const buildResult = spawnSync("pnpm", buildArgs, {
  cwd: packageDir,
  env: resolveDarwinBuildEnv(),
  stdio: "inherit",
});

if (buildResult.status !== 0) {
  process.exit(buildResult.status ?? 1);
}

for (const file of readdirSync(packageDir)) {
  if (file.startsWith("vize-vitrine.") && file.endsWith(".node")) {
    rmSync(path.join(packageDir, file), { force: true });
  }
}

for (const file of readdirSync(outputDir)) {
  if (!file.endsWith(".node")) {
    continue;
  }
  copyFileSync(path.join(outputDir, file), path.join(packageDir, file));
}
