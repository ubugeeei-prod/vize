"""Public npm metadata and SLSA payload binding, without signature verification."""

import base64
import json
import re
import urllib.parse
import urllib.request

REPOSITORY = "https://github.com/ubugeeei-prod/vize"


def public_url(url):
    parsed = urllib.parse.urlsplit(url)
    if (parsed.scheme != "https" or parsed.netloc != "registry.npmjs.org"
            or not parsed.path.startswith("/") or parsed.query or parsed.fragment):
        raise ValueError("public npm registry URL required")
    return url


def read_bytes(url):
    request = urllib.request.Request(public_url(url), headers={
        "User-Agent": "Vize-release-verification/1.0", "Accept": "application/json"})
    with urllib.request.urlopen(request, timeout=30) as response:
        public_url(response.url)
        if response.status != 200:
            raise ValueError("public registry HTTP status differs")
        return response.read()


def read(url):
    return json.loads(read_bytes(url))


def sri_sha512(integrity):
    if not isinstance(integrity, str) or not re.fullmatch(r"sha512-[A-Za-z0-9+/]+={0,2}", integrity):
        raise ValueError("one exact SHA512 SRI required")
    raw = base64.b64decode(integrity[7:], validate=True)
    if len(raw) != 64 or base64.b64encode(raw).decode() != integrity[7:]:
        raise ValueError("canonical SHA512 SRI required")
    return raw


def provenance_payload_binding(dist, source, reader=read):
    url = public_url(dist.get("attestations", {}).get("url", ""))
    expected = sri_sha512(dist.get("integrity")).hex()
    candidates = []
    for item in reader(url).get("attestations", []):
        try:
            envelope = item["bundle"]["dsseEnvelope"]
            payload = json.loads(base64.b64decode(envelope["payload"], validate=True))
        except (KeyError, ValueError, TypeError):
            continue
        if payload.get("predicateType") != "https://slsa.dev/provenance/v1":
            continue
        predicate = payload.get("predicate", {})
        definition = predicate.get("buildDefinition", {})
        workflow = definition.get("externalParameters", {}).get("workflow", {})
        invocation = predicate.get("runDetails", {}).get("metadata", {}).get("invocationId", "")
        commits = [item.get("digest", {}).get("gitCommit")
                   for item in definition.get("resolvedDependencies", [])]
        subjects = [item.get("digest", {}).get("sha512") for item in payload.get("subject", [])]
        matches = (workflow == {"ref": "refs/heads/release/" + source["tag"],
                                "repository": REPOSITORY, "path": ".github/workflows/release.yml"}
                   and commits == [source["H"]] and subjects == [expected]
                   and isinstance(invocation, str) and re.fullmatch(
                       re.escape(REPOSITORY + "/actions/runs/" + source["R"])
                       + r"/attempts/[1-9][0-9]*", invocation) is not None)
        candidates.append({"workflow": workflow, "sourceCommits": commits,
                           "invocation": invocation, "subjectIntegrity": subjects == [expected],
                           "matches": matches})
    if len(candidates) != 1 or not candidates[0]["matches"]:
        raise ValueError("public provenance payload source/run/subject binding differs")
    return {"url": url, "payloads": candidates, "payloadBindingChecked": True,
            "cryptographicSignatureVerificationPerformed": False}


def package_metadata(name, version, entry, source=None, reader=read):
    public_url(entry.get("resolved", ""))
    sri_sha512(entry.get("integrity"))
    metadata = reader("https://registry.npmjs.org/" + urllib.parse.quote(name, safe="") + "/" + version)
    dist = metadata.get("dist", {})
    if (metadata.get("name") != name or metadata.get("version") != version
            or dist.get("integrity") != entry["integrity"]
            or dist.get("tarball") != entry["resolved"]):
        raise ValueError("public registry/lock package identity differs: " + name)
    return (provenance_payload_binding(dist, source, reader) if source else
            {"scope": "third-party Corsa public archive; no Vize source/run provenance"})
