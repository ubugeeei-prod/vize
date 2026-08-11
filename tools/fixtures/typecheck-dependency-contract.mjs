export function installArguments(manager) {
  const argumentsByManager = {
    npm: ["ci", "--ignore-scripts", "--prefer-offline", "--no-audit", "--no-fund"],
    pnpm: ["install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
    yarn: ["install", "--immutable", "--mode=skip-build"],
  };
  const args = argumentsByManager[manager];
  if (args == null) throw new Error(`Unsupported package manager: ${manager}`);
  return args;
}
