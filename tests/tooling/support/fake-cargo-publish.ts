export const assertPublishManifestIsSanitized = [
  "if (args[0] === 'publish') {",
  "  const crateName = args.at(-1);",
  "  const native = path.join(process.cwd(), 'davinci', crateName, 'Cargo.toml');",
  "  const vendor = { vize_oxc_parser: 'oxc_parser', vize_parcel_sourcemap: 'parcel_sourcemap', vize_lightningcss: 'lightningcss', vize_corsa_core: 'corsa_core', vize_corsa_jsonrpc: 'corsa_jsonrpc', vize_corsa_client: 'corsa_client', vize_corsa_lsp: 'corsa_lsp', vize_corsa_orchestrator: 'corsa_orchestrator', vize_corsa: 'corsa' }[crateName];",
  "  const manifestPath = vendor ? path.join(process.cwd(), 'vendor', vendor, 'Cargo.toml') : fs.existsSync(native) ? native : path.join(process.cwd(), 'crates', crateName, 'Cargo.toml');",
  "  const manifest = fs.readFileSync(manifestPath, 'utf8');",
  String.raw`  const devTable = /^\[\s*(?:dev-dependencies\s*(?:\.|\])|target\s*\.\s*(?:"(?:\\.|[^"\\])*"|'[^']*'|[A-Za-z0-9_-]+)\s*\.\s*dev-dependencies\s*(?:\.|\]))/m;`,
  "  if (devTable.test(manifest)) {",
  "    console.error(`publish manifest for ${crateName} still includes dev-dependencies`);",
  "    process.exit(2);",
  "  }",
  "}",
];
