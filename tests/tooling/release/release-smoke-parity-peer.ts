// The peer copies complete fixture trees without Node's recursive cp fast path.
// Fixtures are ordinary files/directories; symbolic links are rejected explicitly.
export const COPY_TREE_SOURCE = String.raw`
function copyTree(source, destination) {
  if (fs.lstatSync(source).isSymbolicLink()) throw new Error("fixture copies forbid symbolic links: " + source);
  const from = fs.realpathSync.native(source);
  let ancestor = path.resolve(destination);
  const missing = [];
  while (!fs.existsSync(ancestor)) {
    missing.unshift(path.basename(ancestor));
    ancestor = path.dirname(ancestor);
  }
  const to = path.join(fs.realpathSync.native(ancestor), ...missing);
  const contains = (parent, child) => {
    const relative = path.relative(parent, child);
    return relative === "" || (!path.isAbsolute(relative) && relative !== ".." && !relative.startsWith(".." + path.sep));
  };
  if (contains(from, to) || contains(to, from)) throw new Error("fixture copy paths must be disjoint");
  function copyEntry(input, output) {
    const stat = fs.lstatSync(input);
    if (stat.isSymbolicLink()) throw new Error("fixture copies forbid symbolic links: " + input);
    if (stat.isDirectory()) {
      fs.mkdirSync(output, { recursive: true });
      for (const name of fs.readdirSync(input).sort()) copyEntry(path.join(input, name), path.join(output, name));
    } else if (stat.isFile()) {
      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.copyFileSync(input, output);
    } else {
      throw new Error("unsupported fixture file type: " + input);
    }
    fs.chmodSync(output, stat.mode & 0o777);
  }
  copyEntry(from, to);
}
`;

// A deterministic npm protocol peer, not a smoke implementation: it records
// exactly what the real driver requests and can corrupt the installed artifact.
export const fakeNpm = String.raw`
const fs = require("node:fs");
const path = require("node:path");
${COPY_TREE_SOURCE}
const argv = process.argv.slice(2);
const cwd = process.cwd();
const fault = process.env.PARITY_FAULT;
const manifest = JSON.parse(fs.readFileSync(path.join(cwd, "package.json"), "utf8"));
fs.appendFileSync(process.env.PARITY_LOG, JSON.stringify({ argv, cwd, manifest }) + "\n");
if (fault === "fail-" + argv[0]) {
  process.stdout.write("npm child stdout\n");
  process.stderr.write("npm child stderr\n");
  process.exit(23);
}
if (argv[0] === "pack") {
  const destination = argv[argv.indexOf("--pack-destination") + 1];
  const count = fault === "zero-packs" ? 0 : fault === "two-packs" ? 2 : 1;
  for (let index = 0; index < count; index++) {
    fs.writeFileSync(path.join(destination, manifest.name.replaceAll("/", "-") + index + ".tgz"), JSON.stringify({ cwd, manifest }));
  }
} else if (argv[0] === "install") {
  for (const [name, specifier] of Object.entries(manifest.dependencies)) {
    if (!specifier.startsWith("file:")) continue;
    const packed = JSON.parse(fs.readFileSync(specifier.slice(5), "utf8"));
    const destination = path.join(cwd, "node_modules", name);
    copyTree(packed.cwd, destination);
    const installed = packed.manifest;
    if (fault === "wrong-name") installed.name = "@vizejs/counterfeit";
    if (fault === "wrong-version") installed.version = "0.0.0-counterfeit";
    if (fault === "installed-workspace") installed.dependencies = { unresolved: "workspace:*" };
    if (fault === "installed-entrypoint") installed.main = "missing-installed.js";
    fs.writeFileSync(path.join(destination, "package.json"), JSON.stringify(installed));
    const bins = typeof installed.bin === "string" ? [installed.name] : Object.keys(installed.bin || {});
    for (const bin of bins) {
      if (fault === "missing-bin") continue;
      const binPath = path.join(cwd, "node_modules/.bin", bin);
      fs.mkdirSync(path.dirname(binPath), { recursive: true });
      fs.writeFileSync(binPath, "#!/bin/sh\nexit 0\n", { mode: fault === "non-executable-bin" ? 0o644 : 0o755 });
    }
  }
} else {
  throw new Error("unexpected npm command " + JSON.stringify(argv));
}
`;
