const audience = process.env.PERMISSION_AUDIENCE;
const versionPath = process.env.VERSION_PATH;
const registry = process.env.JSR_URL.replace(/\/$/, "");
const oidcUrl =
  process.env.ACTIONS_ID_TOKEN_REQUEST_URL + "&audience=" + encodeURIComponent(audience);
const minted = await fetch(oidcUrl, {
  headers: { authorization: "Bearer " + process.env.ACTIONS_ID_TOKEN_REQUEST_TOKEN },
});
if (!minted.ok) throw new Error("oidc " + minted.status);
const { value } = await minted.json();
const authorization = "githuboidc " + value;
const upload = await fetch(registry + versionPath + "?config=/jsr.json", {
  method: "POST",
  headers: { authorization, "content-type": "application/gzip" },
  body: "tarball",
});
await upload.arrayBuffer();
if (!upload.ok) throw new Error("upload " + upload.status);
const provenance = await fetch(registry + versionPath + "/provenance", {
  method: "POST",
  headers: { authorization, "content-type": "application/json" },
  body: JSON.stringify({ bundle: { mediaType: "application/vnd.dev.sigstore.bundle+json" } }),
});
await provenance.arrayBuffer();
console.log("provenance-status=" + provenance.status);
