"""Validate complete observed JSON-RPC frames; never substitute normalized packets."""
from collections import Counter
import hashlib
import json
import pathlib
import re


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise AssertionError("duplicate JSON object key: " + key)
        result[key] = value
    return result


def frames(data):
    offset = 0
    packets = []
    while offset < len(data):
        end = data.find(b"\r\n\r\n", offset)
        assert end >= offset, "truncated raw header"
        headers = data[offset:end].decode("ascii").split("\r\n")
        lengths = [value.split(":", 1)[1].strip() for value in headers if value.lower().startswith("content-length:")]
        assert len(lengths) == 1 and re.fullmatch(r"[0-9]+", lengths[0])
        length = int(lengths[0])
        start = end + 4
        assert start + length <= len(data), "truncated raw body"
        body = data[start:start + length]
        packet = json.loads(body.decode("utf8", "strict"), object_pairs_hook=unique_object)
        assert isinstance(packet, dict), "raw JSON-RPC envelope is not an object"
        packets.append(packet)
        offset = start + length
    assert offset == len(data)
    return packets


SDK_IMAGES = {
    "7.0.2":"4f2de678286401759b3fb4475bafe35b8f32b4b3a07d92642bbf37eadc9b34a4",
    "7.1.0-beta":"a183b1956edbe7e7d98f2249858eedf2d8a9790809cc4909db1276e202faaa70",
}
FIXTURE_REF_TAGS = {
    "7.0.2":{"import":277,"function":263},
    "7.1.0-beta":{"import":278,"function":264},
}


def fixture_reference(handle, version, role, image_sha):
    # Fixture observation only. This is NOT an ASTSyntaxKind or origin classifier.
    if SDK_IMAGES.get(version) != image_sha or version not in FIXTURE_REF_TAGS:
        return False
    parts = handle.split(".", 2)
    return len(parts) == 3 and parts[0].isdigit() and parts[1].isdigit() and parts[2].startswith("/") and int(parts[1]) == FIXTURE_REF_TAGS[version][role]


def verify(directory, version):
    directory = pathlib.Path(directory)
    client_bytes = (directory / "client-to-native.wire.bin").read_bytes()
    native_bytes = (directory / "native-to-client.wire.bin").read_bytes()
    client, native = frames(client_bytes), frames(native_bytes)
    requests = [p for p in client if isinstance(p.get("method"), str) and "id" in p]
    responses = [p for p in native if "id" in p and "method" not in p]
    assert len(requests) == len(client) and len(responses) == len(native)
    symbol_methods = ["getSymbolAtPosition","getAliasedSymbol","getTypeOfSymbol","getSymbolOfType"]
    symbol_ids = [p["id"] for p in requests if p["method"] in symbol_methods]
    candidates = {
        "requests":[p for p in requests if p["id"] in symbol_ids],
        "responses":[p for p in responses if p["id"] in symbol_ids],
    }
    (directory / "raw-symbol-candidates.json").write_text(json.dumps(candidates, indent=2)+"\n")
    request_ids, response_ids = [p["id"] for p in requests], [p["id"] for p in responses]
    assert all(type(value) in [int,str] for value in request_ids+response_ids)
    assert len(set(request_ids)) == len(request_ids)
    assert Counter(request_ids) == Counter(response_ids)
    assert all("result" in p and "error" not in p for p in responses)
    by_id = {p["id"]:p for p in responses}
    counts = Counter(p["method"] for p in requests)
    carrier = json.loads((directory / "carrier-receipt.json").read_text())
    assert carrier["backendSha256"] == SDK_IMAGES[version]
    assert len(carrier["phases"]) == 3
    assert [p["syntaxComplete"] for p in carrier["phases"]] == [True,False,True]
    grouped = {}
    selected_requests = [p for p in requests if p["method"] == "getSymbolAtPosition"]
    for request in selected_requests:
        key = json.dumps(request["params"], sort_keys=True)
        grouped.setdefault(key, []).append(request)
    baselines = [group for group in grouped.values() if len(group) == 3]
    assert len(baselines) == 3
    assert counts["getSymbolAtPosition"] == 54
    assert counts["getAliasedSymbol"] == 15
    assert counts["getTypeOfSymbol"] == 3 and counts["getSymbolOfType"] == 3
    assert counts["getSourceFileNames"] == 3
    symbols = []
    for request in requests:
        value = by_id[request["id"]]["result"]
        if request["method"] in ["getSymbolAtPosition","getAliasedSymbol","getSymbolOfType"] and value is not None:
            assert isinstance(value["declarations"],list) and value["declarations"]
            assert all(isinstance(handle,str) and handle for handle in value["declarations"])
            if version == "7.1.0-beta":
                ref = value["reference"]
                assert type(ref["id"]) is int
                if ref["kind"] == 0:
                    assert all(key in ref["file"] for key in ["fileName","path","contentHash","parseOptionsKey","scriptKind","nodeId"])
                else:
                    assert ref["kind"] == 1 and "snapshot" in ref and "project" in ref
            else:
                assert type(value["id"]) in [int,str]
            symbols.append(value)
    controls = []
    for group in baselines:
        values = [by_id[p["id"]]["result"] for p in group]
        assert all(value == values[0] for value in values)
        selected = values[0]
        assert selected["name"] == "actual"
        argument = selected["reference"] if version == "7.1.0-beta" else selected["id"]
        aliases = [p for p in requests if p["method"] == "getAliasedSymbol" and p["params"]["symbol"] == argument and p["params"]["snapshot"] == group[0]["params"]["snapshot"]]
        assert len(aliases) == 1
        target = by_id[aliases[0]["id"]]["result"]
        assert target["name"] == "useTemplateRef"
        for role, symbol in [("import", selected),("function", target)]:
            handle = symbol["declarations"][0]
            assert fixture_reference(handle,version,role,carrier["backendSha256"])
            index,kind,path = handle.split(".",2)
            other = "7.1.0-beta" if version == "7.0.2" else "7.0.2"
            poisoned = ".".join([index,str(FIXTURE_REF_TAGS[other][role]),path])
            assert not fixture_reference(poisoned,version,role,carrier["backendSha256"])
            assert not fixture_reference(".".join([index,"99999",path]),version,role,carrier["backendSha256"])
            assert not fixture_reference(handle,"unknown",role,carrier["backendSha256"])
            assert not fixture_reference(handle,version,role,"0"*64)
            controls.append({"role":role,"actual":handle,"oppositeVersionTag":poisoned,"rejected":True})
    retained = [{"request":p,"response":by_id[p["id"]]} for p in requests if p["method"] in symbol_methods]
    (directory / "full-wire-symbol-packets.json").write_text(json.dumps(retained,indent=2)+"\n")
    assert carrier["forwardedClientBytes"] == len(client_bytes)
    assert carrier["observedNativeStdoutBytes"] == len(native_bytes)
    result = {
        "schema":"vize.binding-origin.native-api-wire.v2","version":version,
        "scope":"test Unix relay to standalone native stdio; product LSP socket and origin host wiring unqualified",
        "actualRequestCounts":dict(counts),"actualResponseCount":len(responses),
        "clientWireBytes":len(client_bytes),"clientWireSha256":hashlib.sha256(client_bytes).hexdigest(),
        "nativeWireBytes":len(native_bytes),"nativeWireSha256":hashlib.sha256(native_bytes).hexdigest(),
        "sameSnapshotRepeatGroups":3,"rawSymbolPacketsRetained":len(retained),
        "versionBoundFixtureRefControls":controls,
        "nodeRefTagIsNotPortableASTSyntaxKind":True,
        "nativeProgramCount":None,"nativeBindCount":None,
        "originClassifierQualified":False,"hostWire":False,"passed":True,
    }
    (directory / "full-wire-receipt.json").write_text(json.dumps(result,indent=2)+"\n")
    return result
