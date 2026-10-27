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


def verify(directory, version):
    directory = pathlib.Path(directory)
    client_bytes = (directory / "client-to-native.wire.bin").read_bytes()
    native_bytes = (directory / "native-to-client.wire.bin").read_bytes()
    client = frames(client_bytes)
    native = frames(native_bytes)
    requests = [p for p in client if isinstance(p.get("method"), str) and "id" in p]
    assert len(requests) == len(client), "classify unexpected client notifications/responses"
    responses = [p for p in native if "id" in p and "method" not in p]
    assert len(responses) == len(native), "classify unexpected peer callbacks/notifications"
    symbol_ids = [p["id"] for p in requests if p["method"] in ["getSymbolAtPosition", "getAliasedSymbol"]]
    candidates = {
        "requests":[p for p in requests if p["id"] in symbol_ids],
        "responses":[p for p in responses if p["id"] in symbol_ids],
    }
    (directory / "raw-symbol-candidates.json").write_text(json.dumps(candidates, indent=2) + "\n")
    request_ids = [p["id"] for p in requests]
    response_ids = [p["id"] for p in responses]
    assert all(type(value) in [int, str] for value in request_ids + response_ids)
    assert len(set(request_ids)) == len(request_ids), "duplicate request ID"
    assert Counter(request_ids) == Counter(response_ids), "lost or duplicate full response"
    by_id = {p["id"]: p for p in responses}
    assert all("result" in p and "error" not in p for p in responses), "native API error"
    counts = Counter(p["method"] for p in requests)
    assert counts["getSymbolAtPosition"] == 3
    assert counts["getAliasedSymbol"] == 1
    assert counts["getSourceFileNames"] == 1
    assert counts["getSourceFile"] == 1
    selected_requests = [p for p in requests if p["method"] == "getSymbolAtPosition"]
    assert all(p["params"] == selected_requests[0]["params"] for p in selected_requests)
    selected = [by_id[p["id"]]["result"] for p in selected_requests]
    assert len(selected) == 3 and all(p == selected[0] for p in selected), "same-snapshot reference changed"
    assert selected[0]["name"] == "actual"
    alias_request, = [p for p in requests if p["method"] == "getAliasedSymbol"]
    target = by_id[alias_request["id"]]["result"]
    assert target["name"] == "useTemplateRef"
    symbols = [selected[0], target]
    for symbol in symbols:
        assert isinstance(symbol["declarations"], list) and symbol["declarations"]
        assert all(isinstance(handle, str) and handle for handle in symbol["declarations"])
        if version == "7.1.0-beta":
            reference = symbol["reference"]
            assert isinstance(reference, dict)
            assert type(reference["id"]) is int
            if reference["kind"] == 0:
                descriptor = reference["file"]
                for key in ["fileName", "path", "contentHash", "parseOptionsKey", "scriptKind", "nodeId"]:
                    assert key in descriptor, "incomplete native file-owner reference"
            else:
                assert reference["kind"] == 1
                assert "snapshot" in reference and "project" in reference
        else:
            assert version == "7.0.2"
            assert type(symbol["id"]) in [int, str]
    expected_argument = selected[0]["reference"] if version == "7.1.0-beta" else selected[0]["id"]
    assert alias_request["params"]["symbol"] == expected_argument, "adapter lost full selected owner"
    retained = [{"request":p, "response":by_id[p["id"]]} for p in requests if p["method"] in ["getSymbolAtPosition", "getAliasedSymbol"]]
    (directory / "full-wire-symbol-packets.json").write_text(json.dumps(retained, indent=2) + "\n")
    receipt = json.loads((directory / "carrier-receipt.json").read_text())
    assert receipt["forwardedClientBytes"] == len(client_bytes)
    assert receipt["observedNativeStdoutBytes"] == len(native_bytes)
    result = {
        "schema":"vize.binding-origin.native-api-wire.v1",
        "version":version,
        "scope":"pre-normalization Unix relay to standalone native stdio; product LSP socket unqualified",
        "actualRequestCounts":dict(counts),
        "actualResponseCount":len(responses),
        "clientWireBytes":len(client_bytes),
        "clientWireSha256":hashlib.sha256(client_bytes).hexdigest(),
        "nativeWireBytes":len(native_bytes),
        "nativeWireSha256":hashlib.sha256(native_bytes).hexdigest(),
        "sameSnapshotSelectedResponses":3,
        "fullReferenceFieldsRetained":True,
        "rawSymbolFields":[sorted(symbol) for symbol in symbols],
        "rawValueDeclarationPresent":["valueDeclaration" in symbol for symbol in symbols],
        "nativeProgramCount":None,
        "nativeBindCount":None,
        "passed":True,
    }
    (directory / "full-wire-receipt.json").write_text(json.dumps(result, indent=2) + "\n")
    return result
