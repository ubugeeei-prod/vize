export const assertPublishManifestIsSanitized = [
  "if (args[0] === 'publish') {",
  "  const crateName = args.at(-1);",
  "  const native = path.join(process.cwd(), 'davinci', crateName, 'Cargo.toml');",
  "  const manifestPath = fs.existsSync(native) ? native : path.join(process.cwd(), 'crates', crateName, 'Cargo.toml');",
  "  const manifest = fs.readFileSync(manifestPath, 'utf8');",
  "  if (manifest.includes('[dev-dependencies]') || manifest.includes('.dev-dependencies]')) {",
  "    console.error(`publish manifest for ${crateName} still includes dev-dependencies`);",
  "    process.exit(2);",
  "  }",
  "}",
];
