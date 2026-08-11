export const typecheckDependencyManagers = [
  {
    name: "npm",
    version: "11.9.0",
    lockfile: "package-lock.json",
    lockfileContents: '{"lockfileVersion":3}\n',
    installArgs: ["ci", "--ignore-scripts", "--prefer-offline", "--no-audit", "--no-fund"],
  },
  {
    name: "pnpm",
    version: "10.0.0",
    lockfile: "pnpm-lock.yaml",
    lockfileContents: "lockfileVersion: '9.0'\n",
    installArgs: ["install", "--frozen-lockfile", "--ignore-scripts", "--prefer-offline"],
  },
  {
    name: "yarn",
    version: "4.9.2",
    lockfile: "yarn.lock",
    lockfileContents: "__metadata:\n  version: 8\n",
    installArgs: ["install", "--immutable", "--mode=skip-build"],
  },
] as const;
