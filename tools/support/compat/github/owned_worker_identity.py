"""Private exact Node test-worker selection; no command lines leave this module."""
import hashlib
import json
import os
from pathlib import Path
import stat
import struct
import subprocess
import time
from urllib.parse import unquote, urlparse

TARGET = "npm/builder/vite-musea/src/variant-bindings-http.test.ts"
PACKAGE = "npm/builder/vite-musea"
RECIPE = "pnpm --dir ../vite build && vp exec tsx --test --test-reporter=spec --test-reporter=tap --test-reporter-destination=stdout --test-reporter-destination=stderr src/*.test.ts src/**/*.test.ts gallery/components/*.test.ts"
NODE_OPTIONS = "--disable-warning=DEP0040 --disable-warning=DEP0169"
HASH_METRICS = {"calls": 0, "bytes": 0, "seconds": 0.0}


def sha256(path):
    started = time.monotonic()
    HASH_METRICS["calls"] += 1
    digest = hashlib.sha256()
    try:
        with Path(path).open("rb") as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b""):
                HASH_METRICS["bytes"] += len(block)
                digest.update(block)
    finally:
        HASH_METRICS["seconds"] += time.monotonic() - started
    return digest.hexdigest()


def file_identity(path):
    path = Path(path)
    before = path.stat()
    if not stat.S_ISREG(before.st_mode):
        raise ValueError("Nonregular physical provider")
    result = {"path": str(path), "device": before.st_dev, "inode": before.st_ino,
              "bytes": before.st_size, "mode": stat.S_IMODE(before.st_mode),
              "mtimeNs": before.st_mtime_ns, "sha256": sha256(path)}
    after = path.stat()
    if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns, before.st_mode) != (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns, after.st_mode):
        raise ValueError("Provider changed while hashing")
    return result


def elf_identity(path):
    with Path(path).open("rb") as stream:
        header = stream.read(64)
    if len(header) != 64 or header[:6] != b"\x7fELF\x02\x01":
        raise ValueError("Physical provider is not little-endian ELF64")
    kind, machine = struct.unpack_from("<HH", header, 16)
    if kind not in [2, 3] or machine != 62:
        raise ValueError("Physical provider is not an x86_64 executable or shared ELF")
    return {"class": 64, "endianness": "little", "machine": machine, "type": kind}


def state(pid, proc=Path("/proc")):
    directory = proc / str(pid)
    body = (directory / "stat").read_text()
    fields = body[body.rindex(")") + 2:].split()
    return {"pid": pid, "parent": int(fields[1]), "startTicks": fields[19],
            "state": fields[0], "uid": directory.stat().st_uid,
            "executable": os.readlink(directory / "exe")}


def same_birth(left, right):
    return all(left[key] == right[key] for key in ["pid", "startTicks", "uid", "executable"])


def lineage(pid, leader, proc=Path("/proc")):
    chain, seen = [], set()
    while pid != leader["pid"]:
        if pid <= 1 or pid in seen or len(chain) >= 64:
            raise ValueError("Worker is not an owned descendant")
        seen.add(pid)
        item = state(pid, proc)
        chain.append(item)
        pid = item["parent"]
    if not chain or not same_birth(state(pid, proc), leader):
        raise ValueError("Leader birth changed or driver selected")
    for item in chain:
        current = state(item["pid"], proc)
        if not same_birth(current, item) or current["parent"] != item["parent"]:
            raise ValueError("Ancestry changed during selection")
    return chain


def bounded_bytes(path, limit=262144):
    with Path(path).open("rb") as stream:
        body = stream.read(limit + 1)
    if len(body) > limit:
        raise ValueError("Private actor profile exceeds bound")
    return body


def target_identity(root):
    target = root / TARGET
    if target.resolve() != target or any(path.is_symlink() for path in target.parents if path != root.parent):
        raise ValueError("Target source has a symlink")
    body = target.read_bytes()
    blob = hashlib.sha1(b"blob " + str(len(body)).encode() + b"\0" + body).hexdigest()
    expected = subprocess.check_output(["git", "--no-replace-objects", "rev-parse", "HEAD:" + TARGET], cwd=root, text=True).strip()
    if blob != expected:
        raise ValueError("Target physical source differs from its Git blob")
    result = file_identity(target)
    result["gitBlob"] = blob
    return result


def preload_files(argv, root):
    files = set()
    for index, arg in enumerate(argv):
        value = None
        for flag in ["--require", "--import"]:
            if arg == flag:
                value = argv[index + 1]
            elif arg.startswith(flag + "="):
                value = arg[len(flag) + 1:]
        if value is None:
            continue
        if value.startswith("file:"):
            parsed = urlparse(value)
            if parsed.netloc or parsed.query or parsed.fragment:
                raise ValueError("Unqualified loader URL")
            value = unquote(parsed.path)
        path = Path(value).resolve(strict=True)
        if path.name not in ["preflight.cjs", "loader.mjs"] or path.parent.name != "dist" or path.parent.parent.name != "tsx" or not path.is_relative_to((root / "node_modules").resolve()):
            raise ValueError("Unknown physical tsx preload")
        files.add(path)
    if {path.name for path in files} != {"preflight.cjs", "loader.mjs"}:
        raise ValueError("Complete tsx preload pair is absent")
    return [file_identity(path) for path in sorted(files)]


def prepare_profile(root, probe, private_path, environment=None):
    package = json.loads((root / PACKAGE / "package.json").read_text())
    if package["scripts"]["test"] != RECIPE:
        raise ValueError("Package runner recipe changed")
    command = ["vp", "exec", "tsx", "--test", "--test-reporter=spec", "--test-reporter=tap",
               "--test-reporter-destination=stdout", "--test-reporter-destination=stderr", str(probe)]
    environment = dict(os.environ) if environment is None else environment
    if environment.get("NODE_OPTIONS", "") != NODE_OPTIONS:
        raise ValueError("Unknown original Node options")
    probe_before = file_identity(probe)
    result = subprocess.run(command, cwd=root / PACKAGE, env=environment,
                            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=10)
    if result.returncode != 0:
        raise ValueError("Private profile probe failed")
    if file_identity(probe) != probe_before:
        raise ValueError("Private authored probe source changed")
    body = json.loads(bounded_bytes(private_path))
    argv = body["argv"]
    if not body["testWorker"] or body["nodeVersion"] != "v24.14.0" or len(argv) < 5 or argv[0] != body["executable"] or argv[-1] != str(probe):
        raise ValueError("Probe did not authenticate an isolated Node test worker")
    executable = file_identity(Path(body["executable"]).resolve(strict=True))
    executable_elf = elf_identity(executable["path"])
    target = target_identity(root)
    preloads = preload_files(argv, root)
    configuration = [file_identity(root / PACKAGE / "package.json"), file_identity(root / PACKAGE / "tsconfig.json")]
    if any((root / PACKAGE / "src" / name).exists() for name in ["tsconfig.json", "jsconfig.json"]):
        raise ValueError("Target has a nearer configuration than the probe cwd")
    # The original shell recipe supplies this relative spelling. Its physical
    # source remains independently authenticated by target_identity().
    argv[-1] = str(Path(TARGET).relative_to(PACKAGE))
    return {"privateArgv": argv, "executable": executable, "executableElf": executable_elf, "target": target,
            "probe": probe_before, "nodeOptionsBinding": {"allowlisted": True,
                "bytes": len(environment["NODE_OPTIONS"]),
                "sha256": hashlib.sha256(environment["NODE_OPTIONS"].encode()).hexdigest()},
            "preloads": preloads, "configuration": configuration,
            "packageRecipeSha256": hashlib.sha256(RECIPE.encode()).hexdigest(),
            "testWorker": True, "nodeVersion": body["nodeVersion"]}


def select(pid, leader, profile, root, proc=Path("/proc"), owner_uid=None):
    chain = lineage(pid, leader, proc)
    actor = chain[0]
    directory = proc / str(pid)
    argv = bounded_bytes(directory / "cmdline", 65536).split(b"\0")
    if argv[-1:] != [b""]:
        raise ValueError("Malformed private command line")
    expected = [value.encode() for value in profile["privateArgv"]]
    if argv[:-1] != expected:
        return None
    environment = bounded_bytes(directory / "environ").split(b"\0")
    markers = [item for item in environment if item.startswith(b"NODE_TEST_CONTEXT=")]
    if markers != [b"NODE_TEST_CONTEXT=child-v8"]:
        raise ValueError("Actor is not an isolated test worker")
    options = [item[len(b"NODE_OPTIONS="):] for item in environment if item.startswith(b"NODE_OPTIONS=")]
    expected_options = profile["nodeOptionsBinding"]
    if len(options) != 1 or options[0] != NODE_OPTIONS.encode() or len(options[0]) != expected_options["bytes"] or hashlib.sha256(options[0]).hexdigest() != expected_options["sha256"]:
        raise ValueError("Worker private Node options differ from the frozen probe")
    owner_uid = os.getuid() if owner_uid is None else owner_uid
    if actor["uid"] != owner_uid or actor["executable"] != profile["executable"]["path"] or Path(os.readlink(directory / "cwd")) != root / PACKAGE:
        raise ValueError("Worker physical owner, executable or cwd differs")
    if file_identity(actor["executable"]) != profile["executable"] or target_identity(root) != profile["target"]:
        raise ValueError("Worker physical source or executable changed")
    if elf_identity(actor["executable"]) != profile["executableElf"]:
        raise ValueError("Worker ELF machine/type changed")
    for item in profile["preloads"]:
        if file_identity(item["path"]) != item:
            raise ValueError("Actual worker preload changed")
    for item in profile["configuration"]:
        if file_identity(item["path"]) != item:
            raise ValueError("Actual package recipe or tsconfig changed")
    current = state(pid, proc)
    if not same_birth(actor, current):
        raise ValueError("Worker birth changed while qualifying")
    return {"actor": actor, "ancestry": chain, "target": profile["target"],
            "executable": profile["executable"], "preloads": profile["preloads"],
            "exactPrivateArgvMatch": True, "isolatedTestWorker": True}


def providers(pid, proc=Path("/proc")):
    maps = (proc / str(pid) / "maps").read_text()
    if len(maps.encode()) > 1048576:
        raise ValueError("Worker mappings exceed bound")
    files = {}
    for line in maps.splitlines():
        fields = line.split(None, 5)
        if len(fields) != 6 or "x" not in fields[1] or not fields[5].startswith("/"):
            continue
        path = Path(fields[5])
        physical = path.stat()
        major, minor = [int(value, 16) for value in fields[3].split(":")]
        if physical.st_ino != int(fields[4]) or (os.major(physical.st_dev), os.minor(physical.st_dev)) != (major, minor):
            raise ValueError("Mapped provider device or inode changed")
        item = file_identity(path)
        if item["inode"] != int(fields[4]) or item["device"] != physical.st_dev:
            raise ValueError("Provider changed after mapping verification")
        item["elf"] = elf_identity(path)
        files[str(path)] = item
    return {"maps": maps, "providers": list(files.values())}
