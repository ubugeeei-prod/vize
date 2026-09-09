export const assertPublishManifestIsSanitized = [
  "if (args[0] === 'publish') {",
  "  const crateName = args.at(-1);",
  "  const manifest = fs.readFileSync(path.join(process.cwd(), 'crates', crateName, 'Cargo.toml'), 'utf8');",
  "  if (manifest.includes('[dev-dependencies]') || manifest.includes('.dev-dependencies]')) {",
  "    console.error(`publish manifest for ${crateName} still includes dev-dependencies`);",
  "    process.exit(2);",
  "  }",
  "}",
];
