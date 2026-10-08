"""The only prospective execution boundary; require genuine successful return."""

import json
import os
import pathlib
import subprocess

from identity import digest, exact_path, reject_overrides


def command(node, arguments, install, environment):
    return subprocess.check_output([str(node), *arguments], env=environment,
                                   cwd=install, text=True, timeout=30).strip()


def verify_journal(journal, configuration, node, pid, corsa_path):
    data = exact_path(str(journal)).read_bytes()
    if not data or not data.endswith(b"\n"):
        raise ValueError("native journal must contain complete records")
    events = [json.loads(line) for line in data.splitlines()]
    if (events[0].get("event") != "initialized"
            or sum(event.get("event") == "initialized" for event in events) != 1
            or events[0].get("nativePath") != configuration["nativePath"]
            or events[0].get("sha256") != configuration["nativeSha256"]
            or events[0].get("installRoot") != configuration["installRoot"]
            or events[0].get("node") != str(node)):
        raise ValueError("native journal initialization differs")
    attempts = 0
    returned = []
    for event in events:
        if (event.get("schema") != "vize-public-native-custody-event-v1"
                or event.get("pid") != pid
                or event.get("event") not in ("initialized", "attempt", "returned", "exit")):
            raise ValueError("native journal rejected/failed/unknown event or PID differs")
        if event.get("expectedNative") is True:
            if (event.get("actualPath") != configuration["nativePath"]
                    or event.get("sha256") != configuration["nativeSha256"]):
                raise ValueError("native journal path/digest differs")
            if event["event"] == "attempt":
                attempts += 1
            if event["event"] == "returned":
                if attempts <= len(returned) or event.get("corsaPath") != corsa_path:
                    raise ValueError("native return lacks its attempt or bundled Corsa route")
                returned.append(event)
    exits = [event for event in events if event.get("event") == "exit"]
    if (len(returned) != 1 or attempts != 1 or len(exits) > 1
            or (exits and (exits[0].get("code") != 0 or events[-1] != exits[0]))):
        raise ValueError("actual native loader successful-return custody missing")
    return returned[0]


def public_probe(node, hook, install, platform_name, cli_bin, journal, version, corsa_path):
    reject_overrides(os.environ)
    environment = dict(os.environ)
    node_digest = digest(node.read_bytes())
    # Values have already been rejected, so no nonempty ambient override is erased.
    information = json.loads(command(node, ["-e", "process.stdout.write(JSON.stringify({version:process.version,platform:process.platform,arch:process.arch}))"], install, environment))
    if information.get("platform") != "darwin" or information.get("arch") != "arm64":
        raise ValueError("this collector requires the genuine Darwin ARM64 host")
    native = exact_path(command(node, ["-e", "process.stdout.write(require.resolve(process.argv[1],{paths:[process.argv[2]]}))", platform_name, str(install)], install, environment),
                        owner=install / "node_modules" / platform_name)
    if native.suffix != ".node":
        raise ValueError("resolved native provider must be the public platform .node file")
    configuration = {"schema": "vize-public-native-custody-v1", "installRoot": str(install),
                     "nativePath": str(native), "nativeSha256": digest(native.read_bytes()),
                     "journalPath": str(journal)}
    environment["VIZE_PUBLIC_NATIVE_CUSTODY"] = json.dumps(configuration)
    process = subprocess.Popen([str(node), "--require", str(hook), str(cli_bin), "--version"],
                               env=environment, cwd=install, text=True,
                               stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        stdout, stderr = process.communicate(timeout=30)
    except subprocess.TimeoutExpired:
        process.kill()
        process.communicate()
        raise ValueError("actual public CLI version probe timed out") from None
    if process.returncode != 0 or stdout != "vize " + version + "\n" or stderr != "":
        raise ValueError("actual public CLI version failed: " + repr((process.returncode, stdout, stderr)))
    returned = verify_journal(journal, configuration, node, process.pid, str(corsa_path))
    if digest(native.read_bytes()) != configuration["nativeSha256"]:
        raise ValueError("native bytes changed after genuine loader return")
    if digest(node.read_bytes()) != node_digest:
        raise ValueError("explicit Node executable changed during collection")
    return {"node": {"path": str(node), "version": information["version"], "sha256": node_digest},
            "stdout": stdout, "stderr": stderr, "exitCode": process.returncode,
            "native": {"packageName": platform_name, "version": version, "path": str(native),
                       "sha256": configuration["nativeSha256"], "actualLoadedPath": returned["actualPath"],
                       "journalPath": str(journal), "journalSha256": digest(journal.read_bytes()),
                       "successfulReturnObserved": True}}
