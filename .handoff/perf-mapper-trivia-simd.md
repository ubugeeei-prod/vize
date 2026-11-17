# perf-mapper handoff (content mapper profiling track)

Branch `perf/mapper-trivia-simd` is WIP (not PR'd). Status and scripts below.

## SIMD trivia scan (user-approved unsafe, see coordinator message)

Done locally:
- `davinci/vize_l0/src/code_run.rs`: `skip_code` (NEON on aarch64, SSE2 on x86_64, 17-byte windows,
  scalar tail) + `skip_code_scalar`; `#![deny(clippy::undocumented_unsafe_blocks)]`, a `// SAFETY:` on
  every unsafe block (loads + the target_feature boundary call).
- `trivia_scan.rs` now calls `vize_l0::code_run::skip_code` (its local table/skip_code removed).
- Tests (pass natively on aarch64/NEON): every-byte classification, every (byte,next) pair at every
  lane, 20,000-round random-soup differential vs scalar at every start offset (300 rounds under Miri).

Still to do, in order:
1. `cargo test -p vize --lib -- content_mapper` (trivia_scan oracle fuzz) and CI clippy
   (`cargo clippy -p vize_l0 -p vize --profile ci -- -D warnings -D clippy::wildcard_imports`).
2. x86_64 path is untested locally (no x86 std / Miri here: origin toolchain has no nightly). Add to
   `.github/workflows/miri.yml`: `cargo +nightly miri test -p vize_l0 --lib -- code_run` (x86_64 host:
   SSE2 path + scalar) and `cargo +nightly miri test -p vize_l0 --lib --target aarch64-unknown-linux-gnu -- code_run`
   (NEON under Miri). Normal CI test lanes run the x86_64 fuzz natively.
3. Measure: compaction harness (instructions/byte, old 11.6 ins/byte after #13213), whole-stream
   instructions on 4 corpora (scripts below), reply identity (identity.sh), then ONE quiet-runner
   `content-mapper-replay.yml` dispatch and a `level-instruction-counts.yml` dispatch, only after the
   coordinator lifts the CI pause.
4. PR title suggestion: `perf(content-mapper): step over code runs sixteen bytes at a time`.

## Gotchas
- Worktree guard rejects shell commands mixing `cd <worktree>` with scratch paths or words like
  "commit"; put multi-step shell into a script file and run it.
- Build both sides with the SAME toolchain (rustc 1.100.0 since #12954). Cargo 1.100 puts test
  binaries under `target/<profile>/build/<pkg>/<hash>/out/`.
- Local machine is shared (load 30-120): use instructions/cycles from `/usr/bin/time -l`; wall only
  from CI quiet runners.
- `content_mapper.rs` is byte-pinned by producer custody (see `verbatim-offset-successor.ts` in #13243).
- `Setup binding anchor allocation profile` gate forbids any requested-byte increase (killed the
  span_normalize bitset, kept on branch `bench/span-bits-on-1fd0c70`).
- Corpus streams: record with `content-mapper-replay-record.ts`; typecheck-corpora/hosts node_modules
  must be real dirs (not symlinks into scratch) or the recorder fails.

## Scripts (were in the session scratchpad; recreate under a scratch dir `$PM`)

### build.sh

```bash
#!/bin/bash
# usage: build.sh <name> [profile] [extra cargo args...]
# Builds the vize CLI from the worktree into the scratch target dir and copies
# it to bin/vize-<name>. Line tables and no strip, so profilers can symbolize.
set -euo pipefail
WT=/Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-a6e53fe5ab4b6a9bb
PM=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm
name=$1; profile=${2:-ci-opt}; shift; shift || true
upper=$(echo "$profile" | tr 'a-z-' 'A-Z_')
mkdir -p "$PM/bin"
cd "$WT"
export CARGO_TARGET_DIR="$PM/target"
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-8}
export "CARGO_PROFILE_${upper}_DEBUG=line-tables-only"
export "CARGO_PROFILE_${upper}_STRIP=none"
nix develop --command bash -c "cargo build --locked --profile $profile -p vize --bin vize --config 'profile.$profile.package.vize.strip=\"none\"' $* 2>&1 | grep -E '^(error|warning: unused)|Finished' ; cp $PM/target/$profile/vize $PM/bin/vize-$name"
ls -la "$PM/bin/vize-$name"
```

### record.sh

```bash
#!/bin/bash
# Record tsc's content-mapper request streams for the four corpora.
set -euo pipefail
WT=/Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-a6e53fe5ab4b6a9bb
PM=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm
cd "$WT"
node tools/benchmarks/scripts/content-mapper-replay-record.ts \
  --bin "$PM/bin/vize-main" \
  --tsc "$WT/tools/benchmarks/typecheck-hosts/node_modules/.bin/tsc" \
  --corpora "${1:-misskey,n8n,reka-ui,nuxt-ui}" --work "$PM/replay" > "$PM/streams.args"
cat "$PM/streams.args"
```

### tm.py

```python
#!/usr/bin/env python3
"""Alternating timed runs of `vize content-mapper --reply-when-ready` on a
recorded stream, under /usr/bin/time -l (instructions, cycles, RSS, page
reclaims, context switches), with wall measured by perf_counter around it.

usage: tm.py <corpus> <runs> <suffix|-> <bin-name>... [--args 'extra args']
Prints: name wall_s user_s sys_s instructions cycles maxrss reclaims vcsw ivcsw
"""
import os, re, subprocess, sys, time

PM = "/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm"
corpus, runs, suffix, *names = sys.argv[1:]
suffix = "" if suffix == "-" else suffix
mode_args = os.environ.get("MAPPER_ARGS", "--reply-when-ready").split()
stream = f"{PM}/replay/{corpus}{suffix}.bin"
cwd = f"{PM}/replay/{corpus}"

def field(text, pattern):
    m = re.search(pattern, text)
    return m.group(1) if m else "nan"

for _ in range(int(runs)):
    for name in names:
        exe = f"node:{PM}/published/node_modules/vize/bin/vize" if name == "published" else (name if name.startswith("/") or name.startswith("node:") else f"{PM}/bin/vize-{name}")
        if exe.startswith("node:"):
            cmd = ["node", exe[5:], "content-mapper"]
        else:
            cmd = [exe, "content-mapper", *mode_args]
        with open(stream, "rb") as stdin:
            start = time.perf_counter()
            run = subprocess.run(["/usr/bin/time", "-l", *cmd], stdin=stdin,
                                 stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, cwd=cwd)
            wall = time.perf_counter() - start
        err = run.stderr.decode("utf8", "replace")
        if run.returncode != 0:
            print(f"# {name} exited {run.returncode}: {err[-300:]}", file=sys.stderr)
            continue
        user = field(err, r"([\d.]+) user")
        system = field(err, r"([\d.]+) sys")
        ins = field(err, r"(\d+)\s+instructions retired")
        cyc = field(err, r"(\d+)\s+cycles elapsed")
        rss = field(err, r"(\d+)\s+maximum resident set size")
        pr = field(err, r"(\d+)\s+page reclaims")
        vc = field(err, r"(\d+)\s+voluntary context switches")
        ic = field(err, r"(\d+)\s+involuntary context switches")
        print(f"{os.path.basename(name)} {wall:.6f} {user} {system} {ins} {cyc} {rss} {pr} {vc} {ic}", flush=True)
```

### stats.py

```python
#!/usr/bin/env python3
"""Summarize tm.sh lines (name wall user sys ins cyc rss pr vcsw ivcsw) from stdin."""
import statistics, sys, collections

rows = collections.defaultdict(list)
for line in sys.stdin:
    parts = line.split()
    if len(parts) < 10:
        continue
    name = parts[0]
    vals = [float(x) if x not in ("", "-") else float("nan") for x in parts[1:10]]
    rows[name].append(vals)

def q(xs, p):
    xs = sorted(xs)
    if not xs:
        return float("nan")
    k = (len(xs) - 1) * p
    lo = int(k)
    hi = min(lo + 1, len(xs) - 1)
    return xs[lo] + (xs[hi] - xs[lo]) * (k - lo)

print("| build | runs | wall p50 ms | wall p95 ms | CPU p50 s (user+sys) | sys p50 s | instructions p50 | cycles p50 | peak RSS p50 MiB | page reclaims p50 | invol. csw p50 |")
print("|---|---|---|---|---|---|---|---|---|---|---|")
base = None
for name, vals in rows.items():
    wall = [v[0] * 1000 for v in vals]
    cpu = [v[1] + v[2] for v in vals]
    sysv = [v[2] for v in vals]
    ins = [v[3] for v in vals]
    cyc = [v[4] for v in vals]
    rss = [v[5] / 1048576 for v in vals]
    pr = [v[6] for v in vals]
    ic = [v[8] for v in vals]
    m = (q(wall, .5), q(wall, .95), q(cpu, .5), q(sysv, .5), q(ins, .5), q(cyc, .5), q(rss, .5), q(pr, .5), q(ic, .5))
    print(f"| {name} | {len(vals)} | {m[0]:.0f} | {m[1]:.0f} | {m[2]:.3f} | {m[3]:.3f} | {m[4]/1e6:,.1f} M | {m[5]/1e6:,.1f} M | {m[6]:.1f} | {m[7]:,.0f} | {m[8]:,.0f} |")
    if base is None:
        base = m
    else:
        d = lambda i: (m[i] - base[i]) / base[i] * 100 if base[i] else float("nan")
        print(f"| Δ vs first | | {d(0):+.1f}% | {d(1):+.1f}% | {d(2):+.1f}% | {d(3):+.1f}% | {d(4):+.2f}% | {d(5):+.2f}% | {d(6):+.1f}% | {d(7):+.1f}% | {d(8):+.1f}% |")
```

### ab.sh

```bash
#!/run/current-system/sw/bin/bash
# usage: ab.sh <corpus> <runs> <suffix|-> <bin-name>...
# Alternating timed runs, raw lines kept in results/, then the summary table.
PM=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm
corpus=$1; runs=$2; suffix=$3; shift 3
[ "$suffix" = "-" ] && suffix=""
mkdir -p "$PM/results"
raw="$PM/results/$corpus$suffix-$(IFS=_; echo "$*")-$(date +%H%M%S).txt"
python3 "$PM/tm.py" "$corpus" "$runs" "${suffix:--}" "$@" > "$raw"
echo "raw: $raw"
python3 "$PM/stats.py" < "$raw"
```

### suite.sh

```bash
#!/run/current-system/sw/bin/bash
# usage: suite.sh <open-runs> <full-runs> <bin-name>...
# openProject-only sessions and whole tsc streams for the four corpora.
PM=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm
open=$1; full=$2; shift 2
for corpus in misskey n8n reka-ui nuxt-ui; do
  if [ "$open" != 0 ]; then echo "## $corpus openProject session"; "$PM/ab.sh" "$corpus" "$open" -open "$@"; fi
  if [ "$full" != 0 ]; then echo "## $corpus whole tsc stream"; "$PM/ab.sh" "$corpus" "$full" - "$@"; fi
done
```

### identity.sh

```bash
#!/run/current-system/sw/bin/bash
# usage: identity.sh <base-bin> <head-bin> [runs] [modes]
# Replays the four recorded tsc streams into both builds with the repo's
# replay tool, which fails unless every reply (openProject included) is
# byte-identical across runs and builds.
PM=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm
WT=/Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-a6e53fe5ab4b6a9bb
base=$1; head=$2; runs=${3:-2}; modes=${4:-burst}
out=$PM/identity/$base-$head
mkdir -p "$out"
cd "$WT"
node tools/benchmarks/scripts/content-mapper-replay.ts $(cat "$PM/streams.args") \
  --mapper "base=$PM/bin/vize-$base" --mapper "head=$PM/bin/vize-$head" --ready \
  --runs "$runs" --modes "$modes" --out "$out" 2>&1 | tail -3
grep -E '^### |Replies|byte' "$out/replay.md"
```

### headstream.py

```python
#!/usr/bin/env python3
"""Write the frames of a recorded stream up to and including the first
`openProject` (or the first N frames) to a new file.

usage: headstream.py <in.bin> <out.bin> [n]"""
import json, re, sys

data = open(sys.argv[1], "rb").read()
limit = int(sys.argv[3]) if len(sys.argv) > 3 else None
out = bytearray()
at = 0
count = 0
methods = []
while at < len(data):
    end = data.index(b"\r\n\r\n", at)
    length = int(re.search(rb"Content-Length:\s*(\d+)", data[at:end], re.I).group(1))
    body = data[end + 4 : end + 4 + length]
    out += data[at : end + 4 + length]
    at = end + 4 + length
    count += 1
    method = json.loads(body).get("method")
    methods.append(method)
    if limit is None and method == "openProject":
        break
    if limit is not None and count >= limit:
        break
open(sys.argv[2], "wb").write(out)
print(count, methods[:5])
```

### emitstream.py

```python
#!/usr/bin/env python3
"""Rewrite a recorded stream so openProject says noEmit: false (no trivia
compaction), for extracting the uncompacted check projections.

usage: emitstream.py <in.bin> <out.bin>"""
import json, re, sys

data = open(sys.argv[1], "rb").read()
out = bytearray()
at = 0
while at < len(data):
    end = data.index(b"\r\n\r\n", at)
    length = int(re.search(rb"Content-Length:\s*(\d+)", data[at:end], re.I).group(1))
    body = data[end + 4 : end + 4 + length]
    at = end + 4 + length
    msg = json.loads(body)
    if msg.get("method") == "openProject":
        msg["params"].setdefault("compilerOptions", {})["noEmit"] = False
        body = json.dumps(msg, separators=(",", ":")).encode()
    out += b"Content-Length: %d\r\n\r\n" % len(body) + body
open(sys.argv[2], "wb").write(out)
```

### extract.py

```python
#!/usr/bin/env python3
"""Run a mapper on a stream and write each transform reply's text and spans
as a binary corpus for the trivia bench:
  per file: u64 text_len, text bytes, u64 span_count, span_count*6 u64
usage: extract.py <vize-bin> <corpus> <stream-suffix> <out.corpus>"""
import json, re, struct, subprocess, sys

PM = "/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm"
binary, corpus, suffix, out_path = sys.argv[1:5]
with open(f"{PM}/replay/{corpus}{suffix}.bin", "rb") as stdin:
    run = subprocess.run([binary, "content-mapper"], stdin=stdin, capture_output=True, cwd=f"{PM}/replay/{corpus}")
data = run.stdout
at = 0
files = 0
with open(out_path, "wb") as out:
    while at < len(data):
        end = data.index(b"\r\n\r\n", at)
        length = int(re.search(rb"Content-Length:\s*(\d+)", data[at:end], re.I).group(1))
        body = data[end + 4 : end + 4 + length]
        at = end + 4 + length
        msg = json.loads(body)
        result = msg.get("result")
        if not isinstance(result, dict) or "text" not in result:
            continue
        text = result["text"].encode("utf8")
        spans = result.get("mappings", [])
        out.write(struct.pack("<Q", len(text)))
        out.write(text)
        out.write(struct.pack("<Q", len(spans)))
        for span in spans:
            row = list(span) + [(1 << 20) - 1] * (6 - len(span))
            out.write(struct.pack("<6Q", *row[:6]))
        files += 1
print(files, "files", len(data), "reply bytes")
```

### prof.sh

```bash
#!/run/current-system/sw/bin/bash
# usage: prof.sh <bin-name> <corpus> <iterations> [mode]
# Profiles `vize content-mapper` answering a recorded tsc stream with samply.
# mode: burst (default, --reply-when-ready like the launcher) | serial-free
set -euo pipefail
PM=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm
bin=$PM/bin/vize-$1; corpus=$2; n=${3:-5}
out=$PM/prof/$1-$corpus${SUFFIX:-}.json.gz
mkdir -p "$PM/prof"
cd "$PM/replay/$corpus"
cat > "$PM/prof/driver-$1-$corpus${SUFFIX:-}.sh" <<EOF
#!/run/current-system/sw/bin/bash
for i in \$(seq 1 $n); do "$bin" content-mapper --reply-when-ready < "$PM/replay/$corpus${SUFFIX:-}.bin" > /dev/null; done
EOF
chmod +x "$PM/prof/driver-$1-$corpus${SUFFIX:-}.sh"
"$PM/samply-dl/samply-aarch64-apple-darwin/samply" record --save-only --unstable-presymbolicate -r 4000 -o "$out" -- "$PM/prof/driver-$1-$corpus${SUFFIX:-}.sh" 2>&1 | tail -2
ls -la "$out"
```

### sam.py

```python
#!/usr/bin/env python3
"""Aggregate a samply profile (.json.gz + .json.syms.json), CPU-weighted.

usage: sam.py <profile.json.gz> <mode> [regex] [n]
modes:
  libs        CPU by library
  self        CPU by leaf function (outer symbol)
  inline      CPU by innermost inlined function at the leaf (file:line too with 'line')
  incl        CPU inclusive by function (any inline depth), unique per sample
  sys         leaf in a system library: attributed to the first vize-main caller
  sysfn       leaf in a system library: by system function name
  match RE    samples with a function matching RE: first caller above it (vize)
  callers RE  inclusive samples under RE: by direct parent function chain (2 levels)
  under RE    self functions within samples that have RE on stack
"""
import bisect, collections, gzip, json, os, re, sys
COUNT = os.environ.get("W") == "count"

path = sys.argv[1]
mode = sys.argv[2] if len(sys.argv) > 2 else "self"
arg = sys.argv[3] if len(sys.argv) > 3 else None
limit = int(sys.argv[4]) if len(sys.argv) > 4 else 45
prof = json.load(gzip.open(path))
syms = json.load(open(path.replace(".json.gz", ".json.syms.json")))
strings = syms["string_table"]
libsyms = {}
known = {}
for entry in syms["data"]:
    table = sorted(entry["symbol_table"], key=lambda s: s["rva"])
    libsyms[entry["debug_name"]] = ([s["rva"] for s in table], table)
    known[entry["debug_name"]] = {a: entry["symbol_table"][i] for a, i in entry.get("known_addresses", [])}

def symbolize(lib, rva):
    """[(function, file:line)] innermost first."""
    found = libsyms.get(lib)
    if not found:
        return [(f"{lib}+0x{rva:x}", "")]
    rvas, table = found
    exact = known.get(lib, {}).get(rva)
    if exact is not None:
        s = exact
        frames = s.get("frames") or [{"function": s["symbol"]}]
        return [(strings[fr["function"]] if fr.get("function") is not None else strings[s["symbol"]],
                 f"{strings[fr['file']].split('/')[-1]}:{fr.get('line')}" if fr.get("file") is not None else "") for fr in frames]
    i = bisect.bisect_right(rvas, rva) - 1
    if i < 0 or rva >= table[i]["rva"] + max(table[i]["size"], 1) + 0:
        if i < 0:
            return [(f"{lib}+0x{rva:x}", "")]
    s = table[i]
    frames = s.get("frames") or [{"function": s["symbol"]}]
    out = []
    for fr in frames:
        fn = strings[fr["function"]] if fr.get("function") is not None else strings[s["symbol"]]
        loc = f"{strings[fr['file']].split('/')[-1]}:{fr.get('line')}" if fr.get("file") is not None else ""
        out.append((fn, loc))
    return out

libs_list = prof["libs"]
total = 0.0
agg = collections.Counter()
for t in prof["threads"]:
    if not t["processName"].startswith(os.environ.get("PROC", "vize")):
        continue
    ft, st, rt, ss = t["frameTable"], t["stackTable"], t["resourceTable"], t["stringArray"]
    fun = t["funcTable"]
    cache = {}
    def frame_info(fi):
        if fi in cache:
            return cache[fi]
        res = fun["resource"][ft["func"][fi]]
        lib = libs_list[rt["lib"][res]]["debugName"] if res is not None and res >= 0 else "?"
        addr = ft["address"][fi]
        info = (lib, symbolize(lib, addr) if addr is not None and addr >= 0 else [("?", "")])
        cache[fi] = info
        return info
    samples = t["samples"]
    weights = samples.get("threadCPUDelta") or [1] * samples["length"]
    for si, stack in enumerate(samples["stack"]):
        w = (1 if (weights[si] or 0) > 0 else 0) if COUNT else (weights[si] or 0)
        if stack is None or w <= 0:
            continue
        total += w
        frames = []
        s = stack
        while s is not None:
            frames.append(frame_info(st["frame"][s]))
            s = st["prefix"][s]
        # frames[0] is the leaf
        leaf_lib, leaf_syms = frames[0]
        if os.environ.get("NOWAIT") and re.search(r"^(__psynch_mutexwait|__psynch_cvwait|semaphore_wait_trap|__semwait_signal|__ulock_wait2?|__workq_kernreturn|mach_msg2_trap|__select|kevent.*)$", leaf_syms[-1][0]):
            total -= w
            continue
        if mode == "libs":
            agg[leaf_lib] += w
        elif mode == "self":
            agg[f"{leaf_syms[-1][0][:150]}  [{leaf_lib}]"] += w
        elif mode == "inline":
            agg[f"{leaf_syms[0][0][:120]}"] += w
        elif mode == "line":
            agg[f"{leaf_syms[0][0][:90]} @ {leaf_syms[0][1]}"] += w
        elif mode == "incl":
            seen = set()
            for _, fs in frames:
                for fn, _ in fs:
                    seen.add(fn[:150])
            for fn in seen:
                agg[fn] += w
        elif mode == "sys":
            if leaf_lib.startswith(os.environ.get("PROC", "vize")):
                continue
            caller = next((fs[0][0] for lib, fs in frames if lib.startswith(os.environ.get("PROC", "vize"))), "?")
            agg[f"{leaf_lib}:{leaf_syms[-1][0][:40]} <- {caller[:110]}"] += w
        elif mode == "sysfn":
            if leaf_lib.startswith(os.environ.get("PROC", "vize")):
                continue
            agg[f"{leaf_lib}:{leaf_syms[-1][0][:100]}"] += w
        elif mode in ("match", "under", "callers", "vcallers"):
            pat = re.compile(arg)
            flat = [(lib, fn) for lib, fs in frames for fn, _ in fs]
            idx = next((i for i, (_, fn) in enumerate(flat) if pat.search(fn)), None)
            if idx is None:
                continue
            if mode == "match":
                caller = next((fn for lib, fn in flat[idx + 1 :] if not pat.search(fn)), "?")
                agg[caller[:150]] += w
            elif mode == "vcallers":
                skip = re.compile(r"^(<?(std|core|alloc)::|<\*const|<&|<std|<core|<alloc|open|read|stat|run_with|default_read|libsystem|try_fold|call_once|\{closure|map<|and_then|unwrap_or|is_ok_and|exists$|is_file$|is_dir$|metadata$|canonicalize$|fold<|next$|get_or_init|initialize|call$|call_once_force)")
                chain = [fn for lib, fn in flat[idx + 1 :] if lib.startswith(os.environ.get("PROC", "vize")) and not skip.search(fn)][:3]
                agg[" <- ".join(c[:60] for c in chain)] += w
            elif mode == "callers":
                depth = int(os.environ.get("DEPTH", "3"))
                chain = [fn for _, fn in flat[idx + 1 : idx + 1 + depth]]
                agg[" <- ".join(c[:70] for c in chain)] += w
            else:
                agg[f"{flat[0][1][:130]} [{flat[0][0]}]"] += w

print(f"total CPU weight {total:.0f}")
for k, v in agg.most_common(limit):
    print(f"{100.0 * v / total:6.2f}%  {k}")
```

### perfagg.py

```python
#!/usr/bin/env python3
"""Aggregate `perf script` output (gz) for the vize process.

usage: perfagg.py <perf-script.txt.gz> [mode] [filter]
modes:
  libc      -- libc samples attributed to the first non-libc caller frame
  kernel    -- kernel samples: syscall/fault entry, and the first user frame
  match RE  -- samples whose stack has a frame matching RE: first user caller
  self      -- top self frames
"""
import gzip, re, sys, collections

path = sys.argv[1]
mode = sys.argv[2] if len(sys.argv) > 2 else "libc"
pat = re.compile(sys.argv[3]) if len(sys.argv) > 3 else None

def samples():
    cur = None
    frames = []
    with gzip.open(path, "rt", errors="replace") as fh:
        for line in fh:
            if not line.strip():
                if cur is not None:
                    yield cur, frames
                cur, frames = None, []
                continue
            if not line.startswith((" ", "\t")):
                head = line.split()
                comm = head[0]
                m = re.search(r"\s(\d+)\s+cycles", line)
                w = int(m.group(1)) if m else 1
                cur = (comm, w)
                continue
            parts = line.strip().split(" ", 1)
            if len(parts) < 2:
                continue
            sym = parts[1]
            dso = sym[sym.rfind("(") + 1 : -1] if sym.endswith(")") else ""
            name = sym[: sym.rfind("(")].strip() if "(" in sym else sym
            name = re.sub(r"\+0x[0-9a-f]+$", "", name)
            frames.append((name, dso))
    if cur is not None:
        yield cur, frames

total = 0
agg = collections.Counter()
for (comm, w), frames in samples():
    if comm == "perf-exec":
        continue
    total += w
    if not frames:
        continue
    leaf_name, leaf_dso = frames[0]
    if mode == "libc":
        if "libc" not in leaf_dso:
            continue
        caller = next((f"{n}" for n, d in frames if "libc" not in d), "?")
        agg[caller[:150]] += w
    elif mode == "kernel":
        if "kernel" not in leaf_dso:
            continue
        entry = "?"
        for n, d in frames:
            if "kernel" in d and (n.startswith("__x64_sys_") or n.startswith("__do_sys") or n in ("asm_exc_page_fault", "exc_page_fault")):
                entry = n
        user = next((n for n, d in frames if "kernel" not in d and "libc" not in d), "?")
        agg[f"{entry} <- {user[:120]}"] += w
    elif mode == "kentry":
        if "kernel" not in leaf_dso:
            continue
        entry = "?"
        for n, d in frames:
            if "kernel" in d and (n.startswith("__x64_sys_") or n in ("asm_exc_page_fault", "asm_sysvec_call_function", "asm_sysvec_apic_timer_interrupt", "asm_sysvec_reschedule_ipi")):
                entry = n
        agg[entry] += w
    elif mode == "match":
        if not any(pat.search(n) for n, _ in frames):
            continue
        user = next((n for n, d in frames if "vize" in d and not pat.search(n)), "?")
        idx = next(i for i, (n, _) in enumerate(frames) if pat.search(n))
        user = next((n for n, d in frames[idx + 1 :] if "vize" in d), "?")
        agg[user[:150]] += w
    elif mode == "self":
        agg[f"{leaf_dso.split('/')[-1]} {leaf_name[:140]}"] += w

print(f"total weight {total}")
for k, v in agg.most_common(40):
    print(f"{100.0 * v / total:6.2f}%  {k}")
```

### strace_cmp.py

```python
#!/usr/bin/env python3
"""Compare strace -c tables of base and head for each corpus.
usage: strace_cmp.py <profiles dir> [corpus...]"""
import os, re, sys

d = sys.argv[1]
corpora = sys.argv[2:] or ["misskey", "n8n", "reka-ui", "nuxt-ui"]

def table(path):
    out = {}
    for line in open(path):
        parts = line.split()
        if len(parts) < 5 or not re.match(r"^[\d.]+$", parts[0]) or parts[-1] == "total":
            continue
        name = parts[-1]
        calls = int(parts[3])
        errors = int(parts[4]) if len(parts) == 6 else 0
        out[name] = (calls, errors, float(parts[1]))
    return out

for corpus in corpora:
    base = table(os.path.join(d, f"{corpus}.base.strace-c.txt"))
    head = table(os.path.join(d, f"{corpus}.head.strace-c.txt"))
    print(f"### {corpus}")
    print("| syscall | base calls (errors) | head calls (errors) | Δ calls |")
    print("|---|---|---|---|")
    for name in sorted(set(base) | set(head), key=lambda n: -max(base.get(n, (0,))[0], head.get(n, (0,))[0])):
        b = base.get(name, (0, 0, 0)); h = head.get(name, (0, 0, 0))
        if max(b[0], h[0]) < 20:
            continue
        print(f"| {name} | {b[0]} ({b[1]}) | {h[0]} ({h[1]}) | {h[0]-b[0]:+d} |")
```

### prbody.py

```python
#!/usr/bin/env python3
"""Insert an addendum file into a PR body before the trailing footer line and
replace the 'numbers added below' placeholder line.
usage: prbody.py <body.md> <addendum.md> <out.md> [placeholder-substring]"""
import sys

body = open(sys.argv[1]).read()
add = open(sys.argv[2]).read()
placeholder = sys.argv[4] if len(sys.argv) > 4 else "numbers added below when it finishes"
lines = body.split("\n")
lines = [l for l in lines if placeholder not in l]
body = "\n".join(lines)
footer = "🤖 Generated with [Claude Code](https://claude.com/claude-code)"
idx = body.rindex(footer)
body = body[:idx].rstrip() + "\n" + add.rstrip() + "\n\n" + footer + "\n"
open(sys.argv[3], "w").write(body)
```

### anchor.sh

```bash
#!/run/current-system/sw/bin/bash
# usage: anchor.sh <label> -- requested bytes/allocations of the setup-binding anchor profile
PM=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/pm
WT=/Users/nishimura/Source/github.com/ubugeeei-prod/vize/.claude/worktrees/agent-a6e53fe5ab4b6a9bb
label=$1
mkdir -p "$PM/anchor/$label"
cd "$WT"; touch "$PM/anchor.stamp"
export CARGO_TARGET_DIR=$PM/target CARGO_BUILD_JOBS=8
nix develop --command bash -c "cargo test --locked --profile ci-opt -p vize_benchmarks --test setup_binding_anchor_profile --no-run 2>&1 | tail -1" >/dev/null 2>&1
bin=$(find $PM/target/ci-opt -name "setup_binding_anchor_profile-*" -type f -perm +111 -newer "$PM/anchor.stamp" | head -1); [ -n "$bin" ] || bin=$(find $PM/target/ci-opt -name "setup_binding_anchor_profile-*" -type f -perm +111 | head -1)
export VIZE_BINDING_PROFILE_INPUT=$WT/tests/_fixtures/differential/typechecker/prepared-type-only-import-facts/Imports.vue.txt
for shape in locals imports; do
  for count in 32 128 512; do
    VIZE_BINDING_PROFILE_SHAPE=$shape VIZE_BINDING_PROFILE_COUNT=$count VIZE_BINDING_PROFILE_MODE=allocation \
      VIZE_BINDING_PROFILE_OUTPUT=$PM/anchor/$label/$shape-$count.json \
      "$bin" --exact complete_binding_anchor_generation_profile >/dev/null 2>&1
    python3 -c "import json,sys; d=json.load(open('$PM/anchor/$label/$shape-$count.json')); print('$label $shape $count', {k:v for k,v in d.items() if isinstance(v,(int,float))})"
  done
done
```
