const fixture = JSON.parse(process.argv[2]);

const files = await import(fixture.filesModule);
try {
  await files.verifyTree(fixture.directory, fixture.manifest);
  process.exit(1);
} catch (error) {
  process.exit(error instanceof files.PayloadError ? 0 : 2);
}
