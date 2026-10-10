import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const helper = fileURLToPath(new URL("./support/pty-command.py", import.meta.url));

function inspect(script) {
  const result = spawnSync(
    "python3",
    [
      "-c",
      `import importlib.util, json, os, socket, sys, tempfile
spec = importlib.util.spec_from_file_location("pty_command", sys.argv[1])
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
${script}`,
      helper,
    ],
    { encoding: "utf8", timeout: 5_000 },
  );
  assert.equal(result.error, undefined, result.stderr);
  assert.equal(result.status, 0, result.stderr);
  return JSON.parse(result.stdout);
}

void test("PTY phases identify Git transport stages without retaining argument values", () => {
  const phases = inspect(`
commands = [
    [b"git", b"-C", b"/private/credential", b"pull", b"origin", b"main"],
    [b"git", b"fetch", b"https://credential@private.invalid/index"],
    [b"git", b"remote-https", b"origin", b"https://credential@private.invalid/index"],
    [b"/usr/lib/git-core/git-remote-https", b"origin", b"private-value"],
    [b"ssh", b"private-host", b"private-command"],
    [b"/private/unknown-credential", b"private-value"],
    [b"moon", b"update"],
    [b"git", b"unknown-operation"],
]
print(json.dumps([module.command_phase(command) for command in commands]))`);
  assert.deepEqual(phases, [
    "git:pull",
    "git:fetch",
    "git:remote-https",
    "git-remote-https",
    "ssh",
    "other",
    "moon:update",
    "git",
  ]);
  assert.doesNotMatch(JSON.stringify(phases), /private|credential@|unknown-operation/);
});

void test("PTY Git phases distinguish global option values from the actual subcommand", () => {
  const phases = inspect(`
commands = [
    [b"git", b"config", b"credential.helper", b"pull"],
    [b"git", b"-C", b"fetch", b"status"],
    [b"git", b"-C", b"fetch", b"config", b"credential.helper", b"pull"],
    [b"git", b"-c", b"credential.helper=fetch", b"config", b"credential.helper", b"pull"],
    [b"git", b"-Cprivate", b"-ccredential.helper=fetch", b"--no-pager", b"pull"],
    [b"git", b"--git-dir=fetch", b"--work-tree", b"pull", b"remote-https"],
    [b"git", b"--namespace=fetch", b"--config-env=credential.helper=PRIVATE_VALUE", b"config"],
    [b"git", b"-C", b"", b"-C", b"private", b"fetch"],
    [b"git", b"-C"],
    [b"git", b"-c", b""],
    [b"git", b"--config-env=fetch", b"pull"],
    [b"git", b"--unknown", b"private", b"pull"],
    [b"git", b"--", b"fetch"],
    [b"git", b"unknown-operation", b"fetch"],
    [b"moon", b"run", b"update"],
]
print(json.dumps([module.command_phase(command) for command in commands]))`);
  assert.deepEqual(phases, [
    "git:config",
    "git",
    "git:config",
    "git:config",
    "git:pull",
    "git:remote-https",
    "git:config",
    "git:fetch",
    "git",
    "git",
    "git",
    "git",
    "git",
    "git",
    "moon:run",
  ]);
  assert.doesNotMatch(JSON.stringify(phases), /private|credential|PRIVATE_VALUE|unknown/);
});

void test("PTY channel scans close their directory at both entry and emitted-channel bounds", () => {
  const observations = inspect(`
original_scandir = module.os.scandir
results = []
for anonymous in (False, True):
    with tempfile.TemporaryDirectory() as root:
        os.mkdir(os.path.join(root, "fd"))
        for index in range(300):
            target = f"pipe:[{index}]" if anonymous else f"/private/credential-{index}"
            os.symlink(target, os.path.join(root, "fd", str(index)))
        count = {"examined": 0, "closed": False}
        class ObservedDirectory:
            def __init__(self, directory):
                self.entries = original_scandir(directory)
            def __enter__(self):
                return self
            def __exit__(self, *_):
                self.entries.close()
                count["closed"] = True
            def __iter__(self):
                return self
            def __next__(self):
                entry = next(self.entries)
                count["examined"] += 1
                return entry
        module.os.scandir = ObservedDirectory
        channels = module.process_channels(root)
        module.os.scandir = original_scandir
        results.append({**count, "channels": len(channels)})
print(json.dumps(results))`);
  assert.deepEqual(observations, [
    { examined: 256, closed: true, channels: 0 },
    { examined: 64, closed: true, channels: 64 },
  ]);
});

void test(
  "PTY Linux evidence relates anonymous pipe and socket owners while excluding file paths",
  { skip: process.platform !== "linux" },
  () => {
    const evidence = inspect(`
read_fd, write_fd = os.pipe()
left, right = socket.socketpair()
with tempfile.NamedTemporaryFile(prefix="private-credential-") as private_file:
    channels = module.process_channels(f"/proc/{os.getpid()}")
    print(json.dumps({
        "channels": channels,
        "pipe": [str(read_fd), str(write_fd)],
        "socket": [str(left.fileno()), str(right.fileno())],
        "privateFile": str(private_file.fileno()),
    }))`);
    const { channels, pipe, socket: sockets, privateFile } = evidence;
    assert.match(channels[pipe[0]], /^pipe:\[\d+\]$/);
    assert.equal(channels[pipe[0]], channels[pipe[1]]);
    for (const fd of sockets) assert.match(channels[fd], /^socket:\[\d+\]$/);
    assert.equal(Object.hasOwn(channels, privateFile), false);
    assert.doesNotMatch(JSON.stringify(channels), /private|credential|\/tmp\//);
  },
);
