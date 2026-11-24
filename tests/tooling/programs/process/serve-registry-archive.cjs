const fakeArchive = Buffer.from("fixture npm archive\n");
const fakeDist = (name, version) => ({
  tarball: `https://registry.npmjs.org/${name}/-/fixture-${version}.tgz`,
  integrity:
    "sha512-" + require("node:crypto").createHash("sha512").update(fakeArchive).digest("base64"),
});
if (new URL(process.argv.at(-1)).pathname.endsWith(".tgz")) {
  if (process.env.NPM_FAKE_URL_LOG)
    require("node:fs").appendFileSync(process.env.NPM_FAKE_URL_LOG, process.argv.at(-1) + "\n");
  process.stdout.write(fakeArchive);
  process.stderr.write("VIZE_HTTP_STATUS:200");
  process.exit(0);
}

module.exports = { fakeDist };
