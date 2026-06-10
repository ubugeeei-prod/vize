import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const slowReleaseTargets = new Set(["x86_64-apple-darwin", "aarch64-pc-windows-msvc"]);

export const slowNativePackageNames = new Map([
  ["x86_64-apple-darwin", "@vizejs/native-darwin-x64"],
  ["aarch64-pc-windows-msvc", "@vizejs/native-win32-arm64-msvc"],
]);

export const slowNapiPackageDirs = new Map([
  ["x86_64-apple-darwin", "darwin-x64"],
  ["aarch64-pc-windows-msvc", "win32-arm64-msvc"],
]);

export const cliReleasePlatforms = [
  {
    host: "blacksmith-12vcpu-macos-15",
    target: "x86_64-apple-darwin",
    archive: "vize-x86_64-apple-darwin.tar.gz",
  },
  {
    host: "blacksmith-12vcpu-macos-15",
    target: "aarch64-apple-darwin",
    archive: "vize-aarch64-apple-darwin.tar.gz",
  },
  {
    host: "blacksmith-32vcpu-windows-2025",
    target: "x86_64-pc-windows-msvc",
    archive: "vize-x86_64-pc-windows-msvc.zip",
  },
  {
    host: "windows-2025",
    target: "aarch64-pc-windows-msvc",
    archive: "vize-aarch64-pc-windows-msvc.zip",
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "x86_64-unknown-linux-gnu",
    archive: "vize-x86_64-unknown-linux-gnu.tar.gz",
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "aarch64-unknown-linux-gnu",
    archive: "vize-aarch64-unknown-linux-gnu.tar.gz",
  },
];

export const nativeReleasePlatforms = [
  {
    host: "blacksmith-12vcpu-macos-15",
    target: "x86_64-apple-darwin",
    cross_compile: false,
  },
  {
    host: "blacksmith-12vcpu-macos-15",
    target: "aarch64-apple-darwin",
    cross_compile: false,
  },
  {
    host: "blacksmith-32vcpu-windows-2025",
    target: "x86_64-pc-windows-msvc",
    cross_compile: false,
  },
  {
    host: "windows-11-arm",
    target: "aarch64-pc-windows-msvc",
    cross_compile: false,
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "x86_64-unknown-linux-gnu",
    cross_compile: false,
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404",
    target: "x86_64-unknown-linux-musl",
    cross_compile: true,
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404-arm",
    target: "aarch64-unknown-linux-gnu",
    cross_compile: false,
  },
  {
    host: "blacksmith-32vcpu-ubuntu-2404-arm",
    target: "aarch64-unknown-linux-musl",
    cross_compile: true,
  },
];

export function parseReleaseVersion(refName) {
  const match = /^v?(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z.-]+)?$/.exec(refName);
  if (!match) {
    throw new Error(`Release tag must look like vMAJOR.MINOR.PATCH[-PRERELEASE], got ${refName}`);
  }

  return {
    major: Number(match[1]),
    minor: Number(match[2]),
    patch: Number(match[3]),
  };
}

export function releasePlatformPlan(refName) {
  const version = parseReleaseVersion(refName);
  const includeSlowPlatforms = version.minor % 5 === 0;
  const isEnabled = (platform) => includeSlowPlatforms || !slowReleaseTargets.has(platform.target);

  return {
    version,
    includeSlowPlatforms,
    skippedTargets: includeSlowPlatforms ? [] : [...slowReleaseTargets],
    cliMatrix: cliReleasePlatforms.filter(isEnabled),
    nativeMatrix: nativeReleasePlatforms.filter(isEnabled),
  };
}

function writeMultilineOutput(name, value) {
  fs.appendFileSync(process.env.GITHUB_OUTPUT, `${name}<<JSON\n${value}\nJSON\n`);
}

function writeGithubOutputs(refName) {
  const plan = releasePlatformPlan(refName);
  const outputs = {
    include_slow_platforms: String(plan.includeSlowPlatforms),
    release_minor: String(plan.version.minor),
    cli_matrix: JSON.stringify(plan.cliMatrix),
    native_matrix: JSON.stringify(plan.nativeMatrix),
  };

  fs.appendFileSync(
    process.env.GITHUB_OUTPUT,
    `include_slow_platforms=${outputs.include_slow_platforms}\nrelease_minor=${outputs.release_minor}\n`,
  );
  writeMultilineOutput("cli_matrix", outputs.cli_matrix);
  writeMultilineOutput("native_matrix", outputs.native_matrix);

  if (process.env.GITHUB_STEP_SUMMARY) {
    const skipped = plan.skippedTargets.length === 0 ? "none" : plan.skippedTargets.join(", ");
    fs.appendFileSync(
      process.env.GITHUB_STEP_SUMMARY,
      [
        "## Release platform cadence",
        "",
        `- Ref: ${refName}`,
        `- Minor version: ${plan.version.minor}`,
        `- Slow targets enabled: ${plan.includeSlowPlatforms ? "yes" : "no"}`,
        `- Skipped targets: ${skipped}`,
        "",
      ].join("\n"),
    );
  }
}

function removeSkippedNativeDeps(packageJson, skippedTargets) {
  if (!packageJson.optionalDependencies) {
    return;
  }

  for (const target of skippedTargets) {
    const packageName = slowNativePackageNames.get(target);
    if (packageName) {
      delete packageJson.optionalDependencies[packageName];
    }
  }
}

function removeSkippedNapiTargets(packageJson, skippedTargets) {
  if (!Array.isArray(packageJson.napi?.targets)) {
    return;
  }

  const skipped = new Set(skippedTargets);
  packageJson.napi.targets = packageJson.napi.targets.filter((target) => !skipped.has(target));
}

function removeSkippedNapiDirs(packageJsonPath, skippedTargets) {
  const packageDir = path.dirname(packageJsonPath);
  for (const target of skippedTargets) {
    const npmTarget = slowNapiPackageDirs.get(target);
    if (!npmTarget) {
      continue;
    }
    fs.rmSync(path.join(packageDir, "npm", npmTarget), { recursive: true, force: true });
  }
}

export function applyReleasePlatformCadence(refName, packageJsonPath) {
  const plan = releasePlatformPlan(refName);
  const resolvedPackageJsonPath = path.resolve(packageJsonPath);
  const packageJson = JSON.parse(fs.readFileSync(resolvedPackageJsonPath, "utf8"));

  if (plan.includeSlowPlatforms) {
    return {
      changed: false,
      skippedTargets: [],
    };
  }

  removeSkippedNativeDeps(packageJson, plan.skippedTargets);
  removeSkippedNapiTargets(packageJson, plan.skippedTargets);
  fs.writeFileSync(resolvedPackageJsonPath, `${JSON.stringify(packageJson, null, 2)}\n`);
  removeSkippedNapiDirs(resolvedPackageJsonPath, plan.skippedTargets);

  return {
    changed: true,
    skippedTargets: plan.skippedTargets,
  };
}

function applyCadence(refName, packageJsonPaths) {
  for (const packageJsonPath of packageJsonPaths) {
    const result = applyReleasePlatformCadence(refName, packageJsonPath);
    const action = result.changed ? "Applied" : "Skipped";
    const skipped = result.skippedTargets.length === 0 ? "none" : result.skippedTargets.join(", ");
    console.log(
      `${action} release platform cadence for ${packageJsonPath}; skipped targets: ${skipped}`,
    );
  }
}

function main() {
  const [command, refName, ...rest] = process.argv.slice(2);
  if (command === "github-output") {
    if (!process.env.GITHUB_OUTPUT) {
      throw new Error("GITHUB_OUTPUT is required for github-output");
    }
    writeGithubOutputs(refName);
    return;
  }
  if (command === "apply-cadence") {
    if (rest.length === 0) {
      throw new Error("apply-cadence requires at least one package.json path");
    }
    applyCadence(refName, rest);
    return;
  }
  if (command === "print") {
    console.log(JSON.stringify(releasePlatformPlan(refName), null, 2));
    return;
  }

  throw new Error(
    "Usage: node tools/github/release-platforms.mjs <github-output|apply-cadence|print> <ref-name> [...package-json]",
  );
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  main();
}
