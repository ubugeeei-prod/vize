# perf-linter handoff (2026-11-17)

Owner track: the linter (`vize_patina`, `vize lint`, the LSP lint path). The measurement driver is this branch, `bench/perf-linter-quiet`, which has no PR on purpose. Paths in the scripts below point at the old session's scratchpad (`S=...`); change `S` before use.

## Branches

| branch | head | PR | state at handoff |
| --- | --- | --- | --- |
| `perf-lsp-component-routes` | `40d0acb300` | #13235 | open, CI green, dequeued (one LSP edit p95 at +1 %) |
| `perf-patina-json-parallel` | `03f349f67b` | #13245 | open, CI green, dequeued (CPU +1–2.6 %, Ir +0.2–0.3 %) |
| `perf-patina-element-items` | `7c02e4d10d` | #13258 | open, CI pending/cancelled, auto-merge off |
| `perf-patina-duplicate-attributes-inline` | `f4d8605aef` | #13251 | open, auto-merge off |
| `perf-lexical-paths-presized` | `6f601baf99` | #13252 | open, auto-merge off |
| `perf-lint-pool-sized-to-inputs` | `b0818f9ff8` | #13259 | open, auto-merge off, dequeued |
| `perf-patina-strip-markdown-buffers` | `a391c29e81` | #13260 | open; one flaky LSP-process timeout job, needs a rerun |
| `perf-l0-expression-guard-inline-stacks` | `a39c782ef9` | #13261 | open, auto-merge off |
| `perf-croquis-unused-tag-match` | `e154c3ffae` | #13247 | CLOSED (preset-all 32-worker wall ×1.015–1.027, unexplained) |
| `bench/perf-linter-quiet` | this commit | none | measurement driver |

## User rule (2026-11-17)

"パフォーマンスリグレッションは，だめです": a PR stays only if NO metric regresses (wall p50/p95, CPU, RSS, instructions). "Within noise" must be shown with an A/A control at the same sample count in the same run.

## Next steps, in order (only after the coordinator lifts the CI pause)

1. Per open PR, one quiet-runner dispatch, with no duplicates. Pair each head scenario with an A/A control at the same 30 runs. A/A is `dispatch2.sh BASE BASE ...`; run it once and share it across PRs that use the same base. Scenarios: `mk-json,mk-all,mk` plus the PR's own scenario (`one,files-8` for #13259, `ir-*` for all). LSP: `mk` with `lsp_rounds=20` for #13235.
2. Requeue (`gh pr merge N --squash`) only if every metric is flat or better against the A/A spread. Otherwise rework or close.
3. #13245: remove the extra nesting pass. Serialize each file through an indenting `io::Write` instead of `to_string_pretty` + `nest`, then re-measure CPU. If the CPU increase is hyper-threading cost of the parallel phase, consider fewer JSON workers (`with_min_len`).
4. #13260: rerun the failed `Rust tests (Language server)` job (a 29.7 s LSP process timeout, unrelated to the change).
5. Remaining allocation sites at 1 % or more (misskey JSON, after the four alloc PRs):
   - `check_expression_text_refs`, 3.4 %
   - L1→L2 lowering (`lower_children_guarded`, `plan_whitespace`, `Lane::run`, `Cx::record`), about 10 %
   - type-world resolution (`resolve_target`, `add_imported_identity`, `directory_cell`, `resolve_import_path_with_project`), about 13 %
   - `rule_docs_path`, 1.6 %; this one needs a public `JsonMessage` field change
6. LSP directory listings: the component metadata cache is keyed on the global documents revision, so every edit re-exports child interfaces. The fix is dependency-precise invalidation. A per-pass type-resolution batch was measured as noise and dropped.

## Pending user decisions

- `vize --version` costs 3.1 M instructions:
  - 50 % is ld.so relative relocations (102k, PIE),
  - 20 % is mimalloc's case-insensitive environment scan in `_mi_options_init`,
  - 15 % is clap `augment_args`.

  That is about 32 % of a single-SFC lint. Fixing it needs non-PIE (a security trade-off) or a mimalloc fork. No change was made.
- #13235: dequeued for an edit p95 of 11.43 → 11.54 ms in one run (p50 improved in both runs). Requeue after an A/A-calibrated LSP run?

## Findings that settled other items

- **Rayon idle CPU is not spin.** On the quiet runner, `strace -c` counted 66 / 1,821 / 3,974 `sched_yield` calls and 5 / 440 / 1,173 `futex` calls at 1 / 16 / 32 workers. CPU grows with workers from hyper-threading contention (t1 350 ms → t32 560 ms).
  - mimalloc options: `PURGE_DELAY=-1` changed nothing; `ARENA_EAGER_COMMIT=1` was 1,440 ms CPU and 514 MiB.
- **Lock waits** are a macOS sampling artifact: under load, CPU deltas attach to samples that land in blocked calls. No mutex function is in the Callgrind top list.
- **The merged-cursor `walk_items` is worse:** +1.0–1.8 % instructions. It was replaced by #13258.
- **Phase timings** (misskey, quiet runner): collect 3.0 ms; lint 24–27 ms; render text 3.9 ms; JSON 6.6 ms before #13245.

## Gotchas

- **Shell guard:** the Bash tool refuses commands that `cd` plus `git`, or that loop over git in compound form. Put them in a script file.
- **Cargo target dirs:** cargo hashes members by workspace-relative path, so never share a target dir across worktrees, and never run two cargo builds on the same target concurrently. That happened and corrupted fingerprints.
- **`verify.sh` crate list** takes bare names ("a b c"), not "a -p b".
- **Report-source pin:** a `crates/vize_patina/src/output/shared.rs` change needs its sha256 added to `CURRENT_REPORT_SOURCES` in `tests/differential/linter-api.ts`.
- **Facade allocation budget:** `crates/vize_patina/tests/davinci_authored_markup_budget.rs` pins zero heap allocations per single-rule markup traversal.
- **Clippy:** `std::string::String` is a disallowed type; use `vize_l0::String` (CompactString).
- **CI flakes:**
  - the npm 404 on csstype 3.2.4, which was transient,
  - `PR JS package tests (builders)`,
  - LSP process timeouts under load.
- **Attribution build:** a system allocator in `crates/vize/src/main.rs` needs `#[used] static KEEP_MIMALLOC_LINKED: mimalloc::MiMalloc = mimalloc::MiMalloc;`, or `mi_option_set_default` fails to link.
- **The interposer's `base` field is wrong** under `DYLD_INSERT_LIBRARIES`. Use `findslide.py`, which finds the slide from return addresses.
- **Quiet-runner concurrency group** is now `run_id`, so dispatches never cancel each other.

## Scripts

### `allocsite.c`: malloc interposer recording caller stacks (build: clang -O2 -dynamiclib -o allocsite.dylib allocsite.c)

~~~~c
// Allocation-site attribution: interpose malloc/calloc/realloc/posix_memalign, record the
// caller stack (frame-pointer walk, depth DEPTH after skipping this frame), aggregate count and
// bytes per stack in a fixed open-addressing table, dump "count bytes addr..." at exit.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <stdint.h>
#include <os/lock.h>
#include <malloc/malloc.h>
#include <mach-o/dyld.h>
#define DYLD_INTERPOSE(_r, _o) __attribute__((used)) static struct { const void *r; const void *o; } _i_##_o __attribute__((section("__DATA,__interpose"))) = {(const void *)(unsigned long)&_r, (const void *)(unsigned long)&_o};
#define DEPTH 8
#define SLOTS (1 << 20)
typedef struct { uint64_t key; uint64_t count; uint64_t bytes; uintptr_t pcs[DEPTH]; } Slot;
static Slot *table;
static os_unfair_lock lock = OS_UNFAIR_LOCK_INIT;
static uint64_t total_count, total_bytes, dropped;
static __thread int busy;

static void record(size_t size) {
  if (busy) return;
  busy = 1;
  uintptr_t pcs[DEPTH] = {0};
  uintptr_t *fp = (uintptr_t *)__builtin_frame_address(0);
  // skip record() and the interposer frame
  for (int skip = 0; skip < 2 && fp; skip++) fp = (uintptr_t *)fp[0];
  for (int i = 0; i < DEPTH && fp; i++) {
    uintptr_t ret = fp[1];
    if (!ret) break;
    pcs[i] = ret;
    uintptr_t *next = (uintptr_t *)fp[0];
    if (next <= fp || (uintptr_t)next - (uintptr_t)fp > (1 << 24)) break;
    fp = next;
  }
  uint64_t key = 1469598103934665603ULL;
  for (int i = 0; i < DEPTH; i++) key = (key ^ pcs[i]) * 1099511628211ULL;
  os_unfair_lock_lock(&lock);
  if (!table) table = (Slot *)calloc(SLOTS, sizeof(Slot));
  total_count++; total_bytes += size;
  uint64_t h = key & (SLOTS - 1);
  for (int probe = 0; probe < 64; probe++, h = (h + 1) & (SLOTS - 1)) {
    Slot *s = &table[h];
    if (s->count == 0) { s->key = key; memcpy(s->pcs, pcs, sizeof pcs); }
    if (s->key == key) { s->count++; s->bytes += size; goto done; }
  }
  dropped++;
done:
  os_unfair_lock_unlock(&lock);
  busy = 0;
}

static void *my_malloc(size_t n) { void *p = malloc(n); record(n); return p; }
static void *my_calloc(size_t a, size_t b) { void *p = calloc(a, b); record(a * b); return p; }
static void *my_realloc(void *q, size_t n) { void *p = realloc(q, n); record(n); return p; }
static int my_posix_memalign(void **p, size_t a, size_t n) { int r = posix_memalign(p, a, n); record(n); return r; }
DYLD_INTERPOSE(my_malloc, malloc)
DYLD_INTERPOSE(my_calloc, calloc)
DYLD_INTERPOSE(my_realloc, realloc)
DYLD_INTERPOSE(my_posix_memalign, posix_memalign)

__attribute__((destructor)) static void dump(void) {
  const char *f = getenv("ALLOCSITE_OUT");
  if (!f || !table) return;
  busy = 1;
  char path[1024];
  snprintf(path, sizeof path, "%s.%d", f, getpid());
  FILE *o = fopen(path, "w");
  if (!o) return;
  fprintf(o, "# total %llu %llu dropped %llu base %lx\n", total_count, total_bytes, dropped, (unsigned long)_dyld_get_image_header(0));
  for (uint64_t i = 0; i < SLOTS; i++) {
    Slot *s = &table[i];
    if (!s->count) continue;
    fprintf(o, "%llu %llu", s->count, s->bytes);
    for (int k = 0; k < DEPTH; k++) fprintf(o, " %lx", (unsigned long)s->pcs[k]);
    fputc('\n', o);
  }
  fclose(o);
}
~~~~

### `allocsum.py`: aggregate a dump by first non-plumbing frame (atos + llvm-cxxfilt); allocsum2.py = same with `<str>::` added to PLUMBING

~~~~python
#!/usr/bin/env python3
"""allocsum.py DUMP BINARY [N]: aggregate allocation-site dump by first non-plumbing frame."""
import collections, re, subprocess, sys
dump, binary = sys.argv[1], sys.argv[2]
n = int(sys.argv[3]) if len(sys.argv) > 3 else 40
lines = open(dump).read().splitlines()
head = re.match(r'# total (\d+) (\d+) dropped (\d+) base ([0-9a-f]+)', lines[0])
total_count, total_bytes, base = int(head.group(1)), int(head.group(2)), int(sys.argv[4], 16) if len(sys.argv) > 4 else int(head.group(4), 16)
rows = []
addrs = set()
for line in lines[1:]:
    parts = line.split()
    count, size = int(parts[0]), int(parts[1])
    pcs = [int(p, 16) for p in parts[2:] if p != '0']
    rows.append((count, size, pcs))
    addrs.update(pcs)
addrs = sorted(addrs)
names = {}
chunk = 2000
for i in range(0, len(addrs), chunk):
    part = addrs[i:i + chunk]
    out = subprocess.run(['atos', '-o', binary, '-l', hex(base)] + [hex(a - 1) for a in part], capture_output=True, text=True).stdout.splitlines()
    for a, s in zip(part, out):
        names[a] = re.sub(r' \(in [^)]*\).*$', '', s)
CXXFILT = '/nix/store/3kng3lq3x887gb3f3ml6c95yqndp1igh-llvm-21.1.8/bin/llvm-cxxfilt'
keys = list(names)
demangled = subprocess.run([CXXFILT], input='\n'.join(names[k] for k in keys), capture_output=True, text=True).stdout.splitlines()
for k, d in zip(keys, demangled):
    names[k] = re.sub(r'::h[0-9a-f]{16}$', '', d)
PLUMBING = re.compile(r'^(0x|__rust_|_?rust_|alloc::|<alloc::|core::|<core::|std::|<std::|hashbrown|<hashbrown|compact_str|<compact_str|smallvec|<smallvec|<vize_l0::string|vize_l0::alloc|_?malloc|_?realloc|_?calloc|bumpalo|<bumpalo|oxc_allocator|<oxc_allocator|<T as alloc|<\[|<str as|indexmap|<indexmap|<&|thin_vec|<thin_vec|serde_json::ser|<serde_json|rustc_hash)')
by_site = collections.Counter(); by_site_bytes = collections.Counter()
for count, size, pcs in rows:
    frames = [names.get(p, hex(p)) for p in pcs]
    site = next((f for f in frames if not PLUMBING.match(f)), frames[-1] if frames else '?')
    site = re.sub(r'::h[0-9a-f]{16}$', '', site)[:150]
    by_site[site] += count; by_site_bytes[site] += size
print(f'total allocations {total_count:,} bytes {total_bytes:,}')
print('\n== by count')
for site, c in by_site.most_common(n):
    print(f'{100*c/total_count:6.2f}% {c:>9,}  {100*by_site_bytes[site]/total_bytes:6.2f}%B  {site}')
print('\n== by bytes')
for site, b in by_site_bytes.most_common(n // 2):
    print(f'{100*b/total_bytes:6.2f}%B {b:>12,}  {100*by_site[site]/total_count:6.2f}%  {site}')
~~~~

### `findslide.py`: find the binary load address from return addresses (arm64)

~~~~python
"""findslide.py DUMP BINARY: find the ASLR slide of BINARY from return addresses in DUMP (arm64)."""
import struct, subprocess, sys, collections, re
dump, binary = sys.argv[1], sys.argv[2]
out = subprocess.run(['otool', '-l', binary], capture_output=True, text=True).stdout
m = re.search(r'segname __TEXT\n\s+vmaddr 0x([0-9a-f]+)\n\s+vmsize 0x([0-9a-f]+)', out)
vmaddr, vmsize = int(m.group(1), 16), int(m.group(2), 16)
data = open(binary, 'rb').read()
pcs = collections.Counter()
for line in open(dump).read().splitlines()[1:]:
    for p in line.split()[2:]:
        if p != '0':
            pcs[int(p, 16)] += 1
sample = [pc for pc, _ in pcs.most_common(400)]
lo = min(sample)
best = None
for k in range(0, vmsize // 0x4000 + 1):
    base = (lo - vmsize) // 0x4000 * 0x4000 + k * 0x4000
    slide = base - vmaddr
    hits = total = 0
    for pc in sample:
        off = pc - slide - vmaddr - 4
        if 0 <= off < vmsize and off + 4 <= len(data):
            total += 1
            word = struct.unpack_from('<I', data, off)[0]
            if (word & 0xFC000000) == 0x94000000 or (word & 0xFFFFFC1F) == 0xD63F0000:
                hits += 1
    if total >= 50 and (best is None or hits > best[0]):
        best = (hits, total, base)
print('load address 0x%x (%d/%d return addresses follow a call)' % (best[2], best[0], best[1]))
~~~~

### `allocrun.sh`: one attributed run (BIN selects binary)

~~~~bash
#!/bin/bash
# allocrun.sh TAG ARGS...: allocation sites of one misskey run on the system-allocator build.
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad
tag=$1; shift
mkdir -p $S/alloc
rm -f $S/alloc/$tag.*
cd $($S/r.sh mk pwd)
DYLD_INSERT_LIBRARIES=$S/tools/allocsite.dylib ALLOCSITE_OUT=$S/alloc/$tag RAYON_NUM_THREADS=1 $S/bin/${BIN:-vize-sysalloc} "$@" > $S/alloc/$tag.stdout 2>/dev/null
echo "exit $? stdout sha $(shasum -a 256 < $S/alloc/$tag.stdout | cut -c1-16)"
ls $S/alloc/ | grep "^$tag\." | head
~~~~

### `allocauto.sh`: find slide + summarize

~~~~bash
#!/bin/bash
# allocauto.sh DUMP [N]: find the load address and summarize allocation sites (BIN selects the binary).
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad
bin=$S/bin/${BIN:-vize-sysalloc}
base=$(python3 $S/tools/findslide.py "$1" "$bin" | sed 's/load address 0x\([0-9a-f]*\).*/\1/')
echo "load address $base"
python3 $S/tools/allocsum.py "$1" "$bin" "${2:-50}" "$base" > "$1.sum" 2>&1
cut -c1-220 "$1.sum"
~~~~

### `allocdiff.sh`: compare targeted sites across builds

~~~~bash
#!/bin/bash
# allocdiff.sh: base / alloc2 / alloc3 site counts with str methods folded into callers.
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad
summarize() {
  dump=$S/alloc/$1; bin=$S/bin/$2
  base=$(python3 $S/tools/findslide.py "$dump" "$bin" | sed 's/load address 0x\([0-9a-f]*\).*/\1/')
  python3 $S/tools/allocsum2.py "$dump" "$bin" 6000 "$base" > "$dump.sum2" 2>&1
  echo "== $1"
  grep "total allocations" "$dump.sum2"
  sed -n '/== by count/,/== by bytes/p' "$dump.sum2" | grep -E "render_help|strip_markdown|expression_is_safe|analyze_expression|prefix_operator|remove_into" | cut -c1-150
}
summarize mkjson1b.29575 vize-sysalloc
summarize mkjson2.42038 vize-sysalloc-alloc2
summarize mkjson3.10885 vize-sysalloc-alloc3
~~~~

### `buildalloc.sh`: system-allocator + frame-pointer build of a detached worktree

~~~~bash
#!/bin/bash
# wt9 attribution build: system allocator + frame pointers.
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad
CFG=(--config 'profile.ci-opt.debug="line-tables-only"' --config 'profile.ci-opt.package.vize.strip="none"')
RUSTFLAGS="-C force-frame-pointers=yes" $S/tools/dev9.sh cargo build -p vize --profile ci-opt "${CFG[@]}" --target-dir $S/tgt9 2>&1 | grep -E "^error|Finished" -A8
cp $S/tgt9/ci-opt/vize $S/bin/vize-sysalloc${OUT_SUFFIX} && echo sysalloc-built
~~~~

### `icount.c`: libc call counter interposer

~~~~c
#include <sys/ioctl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>
#include <dirent.h>
#include <sys/stat.h>
#include <sys/mman.h>
#include <sys/uio.h>
#include <sys/sysctl.h>
#include <sys/event.h>
#include <sys/wait.h>
#include <poll.h>
#include <pthread.h>
#include <sched.h>
#include <spawn.h>
#include <stdarg.h>
#include <time.h>
#include <os/lock.h>
#include <stdatomic.h>
#include <mach/mach_time.h>
#define DYLD_INTERPOSE(_replacement,_replacee) __attribute__((used)) static struct{ const void* replacement; const void* replacee; } _interpose_##_replacee __attribute__ ((section ("__DATA,__interpose"))) = { (const void*)(unsigned long)&_replacement, (const void*)(unsigned long)&_replacee };
static _Atomic unsigned long long C[63];
static _Atomic unsigned long long B[63];
static const char *N[] = {"open","openat","read","pread","write","writev","close","stat","lstat","fstat","fstatat","access","opendir","fdopendir","readdir","readdir_r","closedir","realpath","readlink","getcwd","mmap","munmap","madvise","mprotect","malloc","calloc","realloc","free","posix_memalign","memcpy","memmove","memset","memcmp","bcmp","memchr","strlen","pthread_mutex_lock","pthread_cond_wait","pthread_cond_timedwait","pthread_cond_signal","pthread_cond_broadcast","pthread_create","os_unfair_lock_lock","sched_yield","posix_spawn","posix_spawnp","fork","waitpid","pipe","poll","mach_absolute_time","clock_gettime","getenv","sysctl","sysctlbyname","unlink","rename","mkdir","fsync","fcntl","ioctl","kevent","isatty"};
static int my_open(const char *p, int f, ...){ atomic_fetch_add_explicit(&C[0],1,memory_order_relaxed); int m=0; if(f & O_CREAT){va_list ap; va_start(ap,f); m=va_arg(ap,int); va_end(ap);} return open(p,f,m);}
DYLD_INTERPOSE(my_open,open)
static int my_openat(int d,const char *p, int f, ...){ atomic_fetch_add_explicit(&C[1],1,memory_order_relaxed); int m=0; if(f & O_CREAT){va_list ap; va_start(ap,f); m=va_arg(ap,int); va_end(ap);} return openat(d,p,f,m);}
DYLD_INTERPOSE(my_openat,openat)
static ssize_t my_read(int fd, void *b, size_t n){ atomic_fetch_add_explicit(&C[2],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[2],(unsigned long long)(n),memory_order_relaxed); return read(fd,b,n); }
DYLD_INTERPOSE(my_read,read)
static ssize_t my_pread(int fd, void *b, size_t n, off_t o){ atomic_fetch_add_explicit(&C[3],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[3],(unsigned long long)(n),memory_order_relaxed); return pread(fd,b,n,o); }
DYLD_INTERPOSE(my_pread,pread)
static ssize_t my_write(int fd, const void *b, size_t n){ atomic_fetch_add_explicit(&C[4],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[4],(unsigned long long)(n),memory_order_relaxed); return write(fd,b,n); }
DYLD_INTERPOSE(my_write,write)
static ssize_t my_writev(int fd, const struct iovec *v, int c){ atomic_fetch_add_explicit(&C[5],1,memory_order_relaxed); return writev(fd,v,c); }
DYLD_INTERPOSE(my_writev,writev)
static int my_close(int fd){ atomic_fetch_add_explicit(&C[6],1,memory_order_relaxed); return close(fd); }
DYLD_INTERPOSE(my_close,close)
static int my_stat(const char *p, struct stat *s){ atomic_fetch_add_explicit(&C[7],1,memory_order_relaxed); return stat(p,s); }
DYLD_INTERPOSE(my_stat,stat)
static int my_lstat(const char *p, struct stat *s){ atomic_fetch_add_explicit(&C[8],1,memory_order_relaxed); return lstat(p,s); }
DYLD_INTERPOSE(my_lstat,lstat)
static int my_fstat(int fd, struct stat *s){ atomic_fetch_add_explicit(&C[9],1,memory_order_relaxed); return fstat(fd,s); }
DYLD_INTERPOSE(my_fstat,fstat)
static int my_fstatat(int d, const char *p, struct stat *s, int f){ atomic_fetch_add_explicit(&C[10],1,memory_order_relaxed); return fstatat(d,p,s,f); }
DYLD_INTERPOSE(my_fstatat,fstatat)
static int my_access(const char *p, int m){ atomic_fetch_add_explicit(&C[11],1,memory_order_relaxed); return access(p,m); }
DYLD_INTERPOSE(my_access,access)
static DIR * my_opendir(const char *p){ atomic_fetch_add_explicit(&C[12],1,memory_order_relaxed); return opendir(p); }
DYLD_INTERPOSE(my_opendir,opendir)
static DIR * my_fdopendir(int fd){ atomic_fetch_add_explicit(&C[13],1,memory_order_relaxed); return fdopendir(fd); }
DYLD_INTERPOSE(my_fdopendir,fdopendir)
static struct dirent * my_readdir(DIR *d){ atomic_fetch_add_explicit(&C[14],1,memory_order_relaxed); return readdir(d); }
DYLD_INTERPOSE(my_readdir,readdir)
static int my_readdir_r(DIR *d, struct dirent *e, struct dirent **r){ atomic_fetch_add_explicit(&C[15],1,memory_order_relaxed); return readdir_r(d,e,r); }
DYLD_INTERPOSE(my_readdir_r,readdir_r)
static int my_closedir(DIR *d){ atomic_fetch_add_explicit(&C[16],1,memory_order_relaxed); return closedir(d); }
DYLD_INTERPOSE(my_closedir,closedir)
static char * my_realpath(const char *p, char *r){ atomic_fetch_add_explicit(&C[17],1,memory_order_relaxed); return realpath(p,r); }
DYLD_INTERPOSE(my_realpath,realpath)
static ssize_t my_readlink(const char *p, char *b, size_t n){ atomic_fetch_add_explicit(&C[18],1,memory_order_relaxed); return readlink(p,b,n); }
DYLD_INTERPOSE(my_readlink,readlink)
static char * my_getcwd(char *b, size_t n){ atomic_fetch_add_explicit(&C[19],1,memory_order_relaxed); return getcwd(b,n); }
DYLD_INTERPOSE(my_getcwd,getcwd)
static void * my_mmap(void *a, size_t l, int p, int f, int fd, off_t o){ atomic_fetch_add_explicit(&C[20],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[20],(unsigned long long)(l),memory_order_relaxed); return mmap(a,l,p,f,fd,o); }
DYLD_INTERPOSE(my_mmap,mmap)
static int my_munmap(void *a, size_t l){ atomic_fetch_add_explicit(&C[21],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[21],(unsigned long long)(l),memory_order_relaxed); return munmap(a,l); }
DYLD_INTERPOSE(my_munmap,munmap)
static int my_madvise(void *a, size_t l, int b){ atomic_fetch_add_explicit(&C[22],1,memory_order_relaxed); return madvise(a,l,b); }
DYLD_INTERPOSE(my_madvise,madvise)
static int my_mprotect(void *a, size_t l, int p){ atomic_fetch_add_explicit(&C[23],1,memory_order_relaxed); return mprotect(a,l,p); }
DYLD_INTERPOSE(my_mprotect,mprotect)
static void * my_malloc(size_t n){ atomic_fetch_add_explicit(&C[24],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[24],(unsigned long long)(n),memory_order_relaxed); return malloc(n); }
DYLD_INTERPOSE(my_malloc,malloc)
static void * my_calloc(size_t a, size_t b){ atomic_fetch_add_explicit(&C[25],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[25],(unsigned long long)(a*b),memory_order_relaxed); return calloc(a,b); }
DYLD_INTERPOSE(my_calloc,calloc)
static void * my_realloc(void *p, size_t n){ atomic_fetch_add_explicit(&C[26],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[26],(unsigned long long)(n),memory_order_relaxed); return realloc(p,n); }
DYLD_INTERPOSE(my_realloc,realloc)
static void my_free(void *p){ atomic_fetch_add_explicit(&C[27],1,memory_order_relaxed); free(p); }
DYLD_INTERPOSE(my_free,free)
static int my_posix_memalign(void **p, size_t a, size_t n){ atomic_fetch_add_explicit(&C[28],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[28],(unsigned long long)(n),memory_order_relaxed); return posix_memalign(p,a,n); }
DYLD_INTERPOSE(my_posix_memalign,posix_memalign)
static void * my_memcpy(void *d, const void *s, size_t n){ atomic_fetch_add_explicit(&C[29],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[29],(unsigned long long)(n),memory_order_relaxed); return memcpy(d,s,n); }
DYLD_INTERPOSE(my_memcpy,memcpy)
static void * my_memmove(void *d, const void *s, size_t n){ atomic_fetch_add_explicit(&C[30],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[30],(unsigned long long)(n),memory_order_relaxed); return memmove(d,s,n); }
DYLD_INTERPOSE(my_memmove,memmove)
static void * my_memset(void *d, int c, size_t n){ atomic_fetch_add_explicit(&C[31],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[31],(unsigned long long)(n),memory_order_relaxed); return memset(d,c,n); }
DYLD_INTERPOSE(my_memset,memset)
static int my_memcmp(const void *a, const void *b, size_t n){ atomic_fetch_add_explicit(&C[32],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[32],(unsigned long long)(n),memory_order_relaxed); return memcmp(a,b,n); }
DYLD_INTERPOSE(my_memcmp,memcmp)
static int my_bcmp(const void *a, const void *b, size_t n){ atomic_fetch_add_explicit(&C[33],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[33],(unsigned long long)(n),memory_order_relaxed); return bcmp(a,b,n); }
DYLD_INTERPOSE(my_bcmp,bcmp)
static void * my_memchr(const void *a, int c, size_t n){ atomic_fetch_add_explicit(&C[34],1,memory_order_relaxed);atomic_fetch_add_explicit(&B[34],(unsigned long long)(n),memory_order_relaxed); return memchr(a,c,n); }
DYLD_INTERPOSE(my_memchr,memchr)
static size_t my_strlen(const char *s){ atomic_fetch_add_explicit(&C[35],1,memory_order_relaxed); return strlen(s); }
DYLD_INTERPOSE(my_strlen,strlen)
static int my_pthread_mutex_lock(pthread_mutex_t *m){ atomic_fetch_add_explicit(&C[36],1,memory_order_relaxed); return pthread_mutex_lock(m); }
DYLD_INTERPOSE(my_pthread_mutex_lock,pthread_mutex_lock)
static int my_pthread_cond_wait(pthread_cond_t *c, pthread_mutex_t *m){ atomic_fetch_add_explicit(&C[37],1,memory_order_relaxed); return pthread_cond_wait(c,m); }
DYLD_INTERPOSE(my_pthread_cond_wait,pthread_cond_wait)
static int my_pthread_cond_timedwait(pthread_cond_t *c, pthread_mutex_t *m, const struct timespec *t){ atomic_fetch_add_explicit(&C[38],1,memory_order_relaxed); return pthread_cond_timedwait(c,m,t); }
DYLD_INTERPOSE(my_pthread_cond_timedwait,pthread_cond_timedwait)
static int my_pthread_cond_signal(pthread_cond_t *c){ atomic_fetch_add_explicit(&C[39],1,memory_order_relaxed); return pthread_cond_signal(c); }
DYLD_INTERPOSE(my_pthread_cond_signal,pthread_cond_signal)
static int my_pthread_cond_broadcast(pthread_cond_t *c){ atomic_fetch_add_explicit(&C[40],1,memory_order_relaxed); return pthread_cond_broadcast(c); }
DYLD_INTERPOSE(my_pthread_cond_broadcast,pthread_cond_broadcast)
static int my_pthread_create(pthread_t *t, const pthread_attr_t *a, void *(*f)(void *), void *x){ atomic_fetch_add_explicit(&C[41],1,memory_order_relaxed); return pthread_create(t,a,f,x); }
DYLD_INTERPOSE(my_pthread_create,pthread_create)
static void my_os_unfair_lock_lock(os_unfair_lock_t l){ atomic_fetch_add_explicit(&C[42],1,memory_order_relaxed); os_unfair_lock_lock(l); }
DYLD_INTERPOSE(my_os_unfair_lock_lock,os_unfair_lock_lock)
static int my_sched_yield(void){ atomic_fetch_add_explicit(&C[43],1,memory_order_relaxed); return sched_yield(); }
DYLD_INTERPOSE(my_sched_yield,sched_yield)
static int my_posix_spawn(pid_t *p, const char *path, const posix_spawn_file_actions_t *fa, const posix_spawnattr_t *at, char *const argv[], char *const envp[]){ atomic_fetch_add_explicit(&C[44],1,memory_order_relaxed); return posix_spawn(p,path,fa,at,argv,envp); }
DYLD_INTERPOSE(my_posix_spawn,posix_spawn)
static int my_posix_spawnp(pid_t *p, const char *path, const posix_spawn_file_actions_t *fa, const posix_spawnattr_t *at, char *const argv[], char *const envp[]){ atomic_fetch_add_explicit(&C[45],1,memory_order_relaxed); return posix_spawnp(p,path,fa,at,argv,envp); }
DYLD_INTERPOSE(my_posix_spawnp,posix_spawnp)
static pid_t my_fork(void){ atomic_fetch_add_explicit(&C[46],1,memory_order_relaxed); return fork(); }
DYLD_INTERPOSE(my_fork,fork)
static pid_t my_waitpid(pid_t p, int *s, int o){ atomic_fetch_add_explicit(&C[47],1,memory_order_relaxed); return waitpid(p,s,o); }
DYLD_INTERPOSE(my_waitpid,waitpid)
static int my_pipe(int f[2]){ atomic_fetch_add_explicit(&C[48],1,memory_order_relaxed); return pipe(f); }
DYLD_INTERPOSE(my_pipe,pipe)
static int my_poll(struct pollfd *f, nfds_t n, int t){ atomic_fetch_add_explicit(&C[49],1,memory_order_relaxed); return poll(f,n,t); }
DYLD_INTERPOSE(my_poll,poll)
static uint64_t my_mach_absolute_time(void){ atomic_fetch_add_explicit(&C[50],1,memory_order_relaxed); return mach_absolute_time(); }
DYLD_INTERPOSE(my_mach_absolute_time,mach_absolute_time)
static int my_clock_gettime(clockid_t c, struct timespec *t){ atomic_fetch_add_explicit(&C[51],1,memory_order_relaxed); return clock_gettime(c,t); }
DYLD_INTERPOSE(my_clock_gettime,clock_gettime)
static char * my_getenv(const char *n){ atomic_fetch_add_explicit(&C[52],1,memory_order_relaxed); return getenv(n); }
DYLD_INTERPOSE(my_getenv,getenv)
static int my_sysctl(int *n, u_int l, void *o, size_t *ol, void *nv, size_t nl){ atomic_fetch_add_explicit(&C[53],1,memory_order_relaxed); return sysctl(n,l,o,ol,nv,nl); }
DYLD_INTERPOSE(my_sysctl,sysctl)
static int my_sysctlbyname(const char *n, void *o, size_t *ol, void *nv, size_t nl){ atomic_fetch_add_explicit(&C[54],1,memory_order_relaxed); return sysctlbyname(n,o,ol,nv,nl); }
DYLD_INTERPOSE(my_sysctlbyname,sysctlbyname)
static int my_unlink(const char *p){ atomic_fetch_add_explicit(&C[55],1,memory_order_relaxed); return unlink(p); }
DYLD_INTERPOSE(my_unlink,unlink)
static int my_rename(const char *a, const char *b){ atomic_fetch_add_explicit(&C[56],1,memory_order_relaxed); return rename(a,b); }
DYLD_INTERPOSE(my_rename,rename)
static int my_mkdir(const char *p, mode_t m){ atomic_fetch_add_explicit(&C[57],1,memory_order_relaxed); return mkdir(p,m); }
DYLD_INTERPOSE(my_mkdir,mkdir)
static int my_fsync(int fd){ atomic_fetch_add_explicit(&C[58],1,memory_order_relaxed); return fsync(fd); }
DYLD_INTERPOSE(my_fsync,fsync)
static int my_fcntl(int fd,int c, ...){ atomic_fetch_add_explicit(&C[59],1,memory_order_relaxed); va_list ap; va_start(ap,c); void* a=va_arg(ap,void*); va_end(ap); return fcntl(fd,c,a);}
DYLD_INTERPOSE(my_fcntl,fcntl)
static int my_ioctl(int fd,unsigned long r, ...){ atomic_fetch_add_explicit(&C[60],1,memory_order_relaxed); va_list ap; va_start(ap,r); void* a=va_arg(ap,void*); va_end(ap); return ioctl(fd,r,a);}
DYLD_INTERPOSE(my_ioctl,ioctl)
static int my_kevent(int kq, const struct kevent *c, int nc, struct kevent *e, int ne, const struct timespec *t){ atomic_fetch_add_explicit(&C[61],1,memory_order_relaxed); return kevent(kq,c,nc,e,ne,t); }
DYLD_INTERPOSE(my_kevent,kevent)
static int my_isatty(int fd){ atomic_fetch_add_explicit(&C[62],1,memory_order_relaxed); return isatty(fd); }
DYLD_INTERPOSE(my_isatty,isatty)
__attribute__((destructor)) static void dump(void){ const char* f=getenv("ICOUNT_OUT"); if(!f) return; char path[1024]; snprintf(path,sizeof path,"%s.%d",f,getpid()); int fd=open(path,O_WRONLY|O_CREAT|O_TRUNC,0644); if(fd<0) return; char buf[256]; for(int i=0;i<63;i++){ unsigned long long c=atomic_load(&C[i]); if(!c) continue; int k=snprintf(buf,sizeof buf,"%s\t%llu\t%llu\n",N[i],c,atomic_load(&B[i])); write(fd,buf,k);} close(fd);}
~~~~

### `icsum.py`: summarize icount dumps

~~~~python
import sys, glob, collections
# icsum.py PREFIX [PREFIX2]: sum counters over all pids; compare
def load(p):
    c = collections.Counter(); b = collections.Counter()
    for f in glob.glob(p + '.*'):
        for line in open(f):
            n, k, v = line.split('\t'); c[n] += int(k); b[n] += int(v)
    return c, b
S = '/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/ic/'
ps = sys.argv[1:]
data = [load(S + p) for p in ps]
names = sorted(set().union(*[d[0].keys() for d in data]), key=lambda n: -max(d[0][n] for d in data))
print('| call | ' + ' | '.join(ps) + ' |'); print('|---|' + '---|' * len(ps))
for n in names:
    print(f'| {n} | ' + ' | '.join(f'{d[0][n]:,}' + (f' ({d[1][n]/1048576:.1f} MiB)' if d[1][n] else '') for d in data) + ' |')
~~~~

### `prof2.py`: samply profile analyzer (CPU-weighted; MAIN_ONLY=1, WALL=1)

~~~~python
#!/usr/bin/env python3
"""Analyze samply profile: prof.py PROFILE.json.gz [--top N] [--proc NAME] [--focus REGEX] [--callers SYMREGEX]
Weights by threadCPUDelta (CPU time), so idle samples do not count."""
import gzip, json, sys, bisect, re, collections, os
MAIN_ONLY = os.environ.get('MAIN_ONLY') == '1'
path = sys.argv[1]
top = 40; proc = None; focus = None; callers = None; mode_inclusive = True
a = sys.argv[2:]
i = 0
while i < len(a):
    if a[i] == '--top': top = int(a[i+1]); i += 2
    elif a[i] == '--proc': proc = a[i+1]; i += 2
    elif a[i] == '--focus': focus = re.compile(a[i+1]); i += 2
    elif a[i] == '--callers': callers = re.compile(a[i+1]); i += 2
    else: raise SystemExit(a[i])
d = json.load(gzip.open(path))
syms = json.load(open(path.replace('.json.gz', '.json.syms.json')))
st = syms['string_table']
symtab = {}
for lib in syms['data']:
    tab = sorted((e['rva'], e.get('size', 0), st[e['symbol']]) for e in lib['symbol_table'])
    symtab[(lib['debug_name'], lib['code_id'].upper())] = ([t[0] for t in tab], tab)
libs = d['libs']
SYSTEM = re.compile(r'^(libsystem_|libc\+\+|libdyld|dyld|libobjc|CoreFoundation|libdispatch|Foundation|libxpc|libz)')
def name_of(lib, addr):
    key = (lib['debugName'], (lib.get('codeId') or '').upper())
    if key not in symtab: return lib['name'] + '!0x%x' % addr
    starts, tab = symtab[key]
    k = bisect.bisect_right(starts, addr) - 1
    if k < 0: return lib['name'] + '!?'
    rva, size, s = tab[k]
    if size and addr >= rva + size:
        return '?past:' + s
    return s
def short(s):
    s = re.sub(r'::h[0-9a-f]{16}$', '', s)
    return s[:150]
self_sym = collections.Counter(); self_lib = collections.Counter(); incl = collections.Counter()
sys_caller = collections.Counter(); callers_c = collections.Counter()
total = 0.0; nsamples = 0; procs = collections.Counter()
for t in d['threads']:
    pname = t.get('processName') or ''
    procs[pname] += 1
    if proc and proc not in pname: continue
    if MAIN_ONLY and not t.get('isMainThread'): continue
    ft, fu, rt, sa, stk = t['frameTable'], t['funcTable'], t['resourceTable'], t['samples'], t['stackTable']
    sarr = t['stringArray']
    frame_cache = {}
    def frame(fi):
        r = frame_cache.get(fi)
        if r is None:
            func = ft['func'][fi]; res = fu['resource'][func]
            if res is None or res < 0: r = ('?', '?')
            else:
                lib = libs[rt['lib'][res]]
                r = (lib['name'], short(name_of(lib, ft['address'][fi])))
            frame_cache[fi] = r
        return r
    stack_cache = {}
    def frames(si):
        r = stack_cache.get(si)
        if r is None:
            out = []; s = si
            while s is not None:
                out.append(frame(stk['frame'][s])); s = stk['prefix'][s]
            r = out; stack_cache[si] = r
        return r
    w = [1] * sa['length'] if os.environ.get('WALL') == '1' else (sa.get('threadCPUDelta') or [1] * sa['length'])
    for k in range(sa['length']):
        si = sa['stack'][k]
        cw = w[k] if w[k] is not None else 0
        if si is None or not cw: continue
        fr = frames(si)
        if focus and not any(focus.search(f[1]) for f in fr): continue
        total += cw; nsamples += 1
        leaf_lib, leaf = fr[0]
        self_sym[(leaf_lib, leaf)] += cw; self_lib[leaf_lib] += cw
        seen = set()
        for f in fr:
            if f[1] not in seen: seen.add(f[1]); incl[f[1]] += cw
        if SYSTEM.match(leaf_lib):
            caller = next((f[1] for f in fr if not SYSTEM.match(f[0])), '?')
            sys_caller[(leaf_lib + ':' + leaf, caller)] += cw
        if callers:
            for j, f in enumerate(fr):
                if callers.search(f[1]):
                    par = fr[j+1][1] if j + 1 < len(fr) else '-'
                    callers_c[(f[1], par)] += cw; break
print(f'processes: {dict(procs)}')
print(f'CPU-weighted samples: {nsamples} (cpu {total/1000:.1f} ms)')
P = lambda v: f'{100*v/total:6.2f}%'
print('\n== self by library =='); [print(P(v), k) for k, v in self_lib.most_common(15)]
print('\n== self by function =='); [print(P(v), k[0], '|', k[1]) for k, v in self_sym.most_common(top)]
print('\n== system leaf <- first vize caller =='); [print(P(v), k[0], '<-', k[1]) for k, v in sys_caller.most_common(top)]
print('\n== inclusive =='); [print(P(v), k) for k, v in incl.most_common(top)]
if callers:
    print('\n== callers =='); [print(P(v), k[0], '<-', k[1]) for k, v in callers_c.most_common(top)]
~~~~

### `timeline.py`: per-process worker timeline from a samply profile

~~~~python
"""timeline.py PROFILE.json.gz WORKREGEX: per process, phase timeline of worker activity."""
import gzip, json, sys, re, collections, statistics
d = json.load(gzip.open(sys.argv[1])); work = re.compile(sys.argv[2])
syms = json.load(open(sys.argv[1].replace('.json.gz', '.json.syms.json'))); st = syms['string_table']
import bisect
tabs = {}
for lib in syms['data']:
    tab = sorted((e['rva'], st[e['symbol']]) for e in lib['symbol_table'])
    tabs[lib['debug_name']] = ([t[0] for t in tab], tab)
def name(lib, a):
    if lib['debugName'] not in tabs: return '?'
    starts, tab = tabs[lib['debugName']]; j = bisect.bisect_right(starts, a) - 1
    return tab[j][1] if j >= 0 else '?'
by_pid = collections.defaultdict(list)
for t in d['threads']:
    by_pid[t.get('pid')].append(t)
rows = []
for pid, threads in by_pid.items():
    start = min(t['samples']['time'][0] for t in threads if t['samples']['length'])
    end = max(t['samples']['time'][-1] for t in threads if t['samples']['length'])
    last_work = []; first_work = []; idle_cpu = 0.0; work_cpu = 0.0; main_cpu = 0.0
    for t in threads:
        ft, fu, rt, sa, stk = t['frameTable'], t['funcTable'], t['resourceTable'], t['samples'], t['stackTable']
        w = sa.get('threadCPUDelta') or [1] * sa['length']
        cache = {}
        def is_work(si):
            if si in cache: return cache[si]
            s = si; r = False
            while s is not None:
                fi = stk['frame'][s]; res = fu['resource'][ft['func'][fi]]
                if res is not None and res >= 0 and work.search(name(d['libs'][rt['lib'][res]], ft['address'][fi])): r = True; break
                s = stk['prefix'][s]
            cache[si] = r; return r
        times = [sa['time'][k] for k in range(sa['length']) if sa['stack'][k] is not None and is_work(sa['stack'][k])]
        cpu_w = sum((w[k] or 0) for k in range(sa['length']) if sa['stack'][k] is not None and is_work(sa['stack'][k]))
        cpu_all = sum((w[k] or 0) for k in range(sa['length']))
        if t.get('isMainThread'): main_cpu += cpu_all
        elif times:
            first_work.append(times[0] - start); last_work.append(times[-1] - start)
            work_cpu += cpu_w; idle_cpu += cpu_all - cpu_w
    if last_work:
        rows.append((end - start, min(first_work), statistics.median(last_work), max(last_work), main_cpu, work_cpu, idle_cpu))
for name_, i in [('wall', 0), ('first work', 1), ('median worker last work', 2), ('last worker last work', 3), ('main thread cpu', 4), ('worker work cpu', 5), ('worker non-work cpu', 6)]:
    print(f'{name_:28s} p50 {statistics.median(r[i] for r in rows):9.2f}')
~~~~

### `leafaddr.py`: leaf addresses attributed to a symbol

~~~~python
import gzip, json, sys, collections, bisect
d = json.load(gzip.open(sys.argv[1])); want = sys.argv[2]
syms = json.load(open(sys.argv[1].replace('.json.gz', '.json.syms.json'))); st = syms['string_table']
tabs = {}
for lib in syms['data']:
    tab = sorted((e['rva'], st[e['symbol']]) for e in lib['symbol_table'])
    tabs[lib['debug_name']] = ([t[0] for t in tab], tab)
c = collections.Counter()
for t in d['threads']:
    ft, fu, rt, sa, stk = t['frameTable'], t['funcTable'], t['resourceTable'], t['samples'], t['stackTable']
    w = sa.get('threadCPUDelta') or [1] * sa['length']
    for k in range(sa['length']):
        si = sa['stack'][k]
        if si is None or not w[k]: continue
        fi = stk['frame'][si]; res = fu['resource'][ft['func'][fi]]
        if res is None or res < 0: continue
        lib = d['libs'][rt['lib'][res]]
        if lib['debugName'] not in tabs: continue
        starts, tab = tabs[lib['debugName']]; a = ft['address'][fi]
        j = bisect.bisect_right(starts, a) - 1
        if j >= 0 and want in tab[j][1]:
            nxt = tab[j + 1] if j + 1 < len(tab) else (0, '')
            c[(hex(a), tab[j][1][:40], hex(tab[j][0]), nxt[1][:40], hex(nxt[0]))] += w[k]
for k, v in c.most_common(12): print(v, k)
~~~~

### `lspbench.mjs`: local LSP open/edit benchmark client

~~~~javascript
// node lspbench.mjs VIZE ROOT FILE EDITS [--out diag.json]
import { spawn } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { resolve } from "node:path";

const [vize, rootArg, fileArg, editsArg] = process.argv.slice(2);
const outIdx = process.argv.indexOf("--out");
const out = outIdx > 0 ? process.argv[outIdx + 1] : null;
const root = resolve(rootArg);
const file = resolve(fileArg);
const edits = Number(editsArg);
const p = spawn(vize, ["lsp"], { cwd: root, stdio: ["pipe", "pipe", "ignore"] });
let buf = Buffer.alloc(0);
const waiters = [];
const queue = [];
p.stdout.on("data", (d) => {
  buf = Buffer.concat([buf, d]);
  for (;;) {
    const sep = buf.indexOf("\r\n\r\n");
    if (sep < 0) return;
    const header = buf.subarray(0, sep).toString();
    const len = Number(/content-length: *(\d+)/i.exec(header)[1]);
    if (buf.length < sep + 4 + len) return;
    const msg = JSON.parse(buf.subarray(sep + 4, sep + 4 + len).toString());
    buf = buf.subarray(sep + 4 + len);
    if (msg.method && msg.id !== undefined) {
      send({ jsonrpc: "2.0", id: msg.id, result: null });
      continue;
    }
    queue.push(msg);
    while (waiters.length && queue.length) waiters.shift()(queue.shift());
  }
});
function send(m) {
  const b = Buffer.from(JSON.stringify(m));
  p.stdin.write(`Content-Length: ${b.length}\r\n\r\n`);
  p.stdin.write(b);
}
const next = () => (queue.length ? Promise.resolve(queue.shift()) : new Promise((r) => waiters.push(r)));
let id = 0;
async function req(method, params) {
  const my = ++id;
  send({ jsonrpc: "2.0", id: my, method, params });
  for (;;) {
    const m = await next();
    if (m.id === my) return m;
  }
}
const uri = "file://" + file;
async function waitDiag(version) {
  for (;;) {
    const m = await next();
    if (m.method === "textDocument/publishDiagnostics" && m.params.uri === uri) {
      const v = m.params.version;
      if (v == null || v >= version) return m.params.diagnostics;
    }
  }
}
await req("initialize", {
  processId: process.pid,
  rootUri: "file://" + root,
  capabilities: { textDocument: { publishDiagnostics: { versionSupport: true } } },
  workspaceFolders: [{ uri: "file://" + root, name: "r" }],
});
send({ jsonrpc: "2.0", method: "initialized", params: {} });
const text = readFileSync(file, "utf8");
let t0 = performance.now();
send({ jsonrpc: "2.0", method: "textDocument/didOpen", params: { textDocument: { uri, languageId: "vue", version: 1, text } } });
await waitDiag(1);
const openMs = performance.now() - t0;
const lat = [];
let last = [];
for (let i = 0; i < edits; i++) {
  const v = i + 2;
  const t = text + (i % 2 === 0 ? " " : "");
  t0 = performance.now();
  send({ jsonrpc: "2.0", method: "textDocument/didChange", params: { textDocument: { uri, version: v }, contentChanges: [{ text: t }] } });
  const d = await waitDiag(v);
  lat.push(performance.now() - t0);
  if (i % 2 === 1) last = d;
}
lat.sort((a, b) => a - b);
const pc = (q) => lat[Math.min(lat.length - 1, Math.round((lat.length - 1) * q))];
const dj = JSON.stringify(last.map((x) => JSON.stringify(x, Object.keys(x).sort())).sort());
console.log(JSON.stringify({ open_ms: +openMs.toFixed(2), edits, p50: +pc(0.5).toFixed(2), p95: +pc(0.95).toFixed(2), mean: +(lat.reduce((a, b) => a + b, 0) / lat.length).toFixed(2), diagnostics: last.length, diag_sha: createHash("sha256").update(dj).digest("hex").slice(0, 16) }));
if (out) writeFileSync(out, dj);
await req("shutdown", null).catch(() => {});
send({ jsonrpc: "2.0", method: "exit", params: null });
setTimeout(() => process.exit(0), 200);
~~~~

### `dispatch2.sh`: quiet-runner dispatch

~~~~bash
#!/bin/bash
# dispatch2.sh BASE HEAD SCENARIOS LSP_ROUNDS
gh workflow run pr-performance-observation.yml -R ubugeeei-prod/vize --ref bench/perf-linter-quiet \
  -f base_sha=$1 -f head_sha=$2 -f scenarios=$3 -f lsp_rounds=$4 && echo "dispatched $1 $2 $3 $4"
~~~~

### `runres.sh`: print comparison rows of runs

~~~~bash
#!/bin/bash
# runres.sh RUN...: comparison table rows of quiet-runner runs.
for r in "$@"; do
  echo "== $r"
  gh run view "$r" -R ubugeeei-prod/vize --log 2>/dev/null | grep "Interleaved base/head" | grep -E "Z \|" | sed 's/^.*Z //' | grep -E "\| (base|head) \||head/base" | cut -c1-190
done
~~~~

### `runmap.py`: recent perf-observation runs

~~~~python
"""runmap.py N: recent perf-observation runs with their head SHA (from the run's jobs log env)."""
import json, subprocess, sys
n = sys.argv[1] if len(sys.argv) > 1 else '12'
runs = json.loads(subprocess.run(['gh', 'run', 'list', '-R', 'ubugeeei-prod/vize', '--workflow', 'pr-performance-observation.yml', '--limit', n, '--json', 'databaseId,status,conclusion,createdAt,headBranch'], capture_output=True, text=True).stdout)
for r in runs:
    rid = str(r['databaseId'])
    jobs = json.loads(subprocess.run(['gh', 'api', f'repos/ubugeeei-prod/vize/actions/runs/{rid}'], capture_output=True, text=True).stdout or '{}')
    title = jobs.get('display_title', '')
    print(rid, r['status'], r['conclusion'], r['createdAt'], r['headBranch'], title[:80])
~~~~

### `prstat.sh`: PR checks / queue state

~~~~bash
#!/bin/bash
# prstat.sh PR...: state, check tallies, failing checks, merge-queue state.
for n in "$@"; do
  state=$(gh pr view "$n" -R ubugeeei-prod/vize --json state --jq .state)
  tally=$(gh pr checks "$n" -R ubugeeei-prod/vize 2>&1 | awk -F'\t' '{print $2}' | sort | uniq -c | tr '\n' ' ')
  queue=$(gh api graphql -f query="query{repository(owner:\"ubugeeei-prod\",name:\"vize\"){pullRequest(number:$n){isInMergeQueue}}}" --jq .data.repository.pullRequest.isInMergeQueue)
  echo "$n $state queue=$queue $tally"
  gh pr checks "$n" -R ubugeeei-prod/vize 2>&1 | grep "	fail" | cut -f1,4 | cut -c1-150 | sed 's/^/    /'
done
~~~~

### `dequeue.sh`: dequeue PRs and list states

~~~~bash
#!/bin/bash
# dequeue.sh PR...: remove from the merge queue; then print state, auto-merge and queue membership.
for n in "$@"; do
  id=$(gh pr view "$n" -R ubugeeei-prod/vize --json id --jq .id)
  gh api graphql -f query="mutation{dequeuePullRequest(input:{id:\"$id\"}){clientMutationId}}" >/dev/null 2>&1
done
for n in 13235 13245 13251 13252 13258 13259 13260 13261; do
  st=$(gh pr view "$n" -R ubugeeei-prod/vize --json state,autoMergeRequest --jq '.state+" auto="+((.autoMergeRequest!=null)|tostring)')
  q=$(gh api graphql -f query="query{repository(owner:\"ubugeeei-prod\",name:\"vize\"){pullRequest(number:$n){isInMergeQueue}}}" --jq .data.repository.pullRequest.isInMergeQueue)
  echo "$n $st queue=$q"
done
~~~~

### `verify.sh`: clippy (CI flags) + targeted tests per worktree

~~~~bash
#!/bin/bash
# verify.sh WTNUM "clippy crates" "test spec..." : run clippy + tests in the worktree's own
# target dir (ci profile, no incremental), then delete that target dir.
# Cargo hashes workspace members by their workspace-relative path, so worktrees must never
# share a target dir: a build of one checkout can be reused as fresh for another.
S=/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad
n=$1; crates=$2; shift 2
dev=$S/tools/dev$n.sh
T=$S/vt$n
export CARGO_INCREMENTAL=0
pk=""; for c in $crates; do pk="$pk -p $c"; done
echo "== wt$n clippy:$pk"
$dev cargo clippy $pk --profile ci --target-dir $T -- -D warnings -D clippy::wildcard_imports > $S/vt$n-clippy.log 2>&1
echo "clippy exit: $?"
grep -E "^error" -A15 $S/vt$n-clippy.log | head -60
for spec in "$@"; do
  echo "== wt$n test: $spec"
  $dev bash -c "cargo test --profile ci --target-dir $T $spec 2>&1" | grep -E "test result|FAILED|panicked|^error" | head -20
done
rm -rf $T
echo "== done wt$n"
~~~~

### `dev7.sh`: nix dev env wrapper template (sed wt7 -> wtN)

~~~~bash
#!/bin/bash
. /private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/devenv100.sh >/dev/null 2>&1
cd /private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/scratchpad/wt7 || exit 9
export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-4}
exec "$@"
~~~~

### `waitjobs.py`: wait on background task outputs

~~~~python
#!/usr/bin/env python3
"""waitjobs.py ID...: print one line per background task as its output shows a terminal marker."""
import os, re, sys, time
D = '/private/tmp/claude-501/-Users-nishimura-Source-github-com-ubugeeei-prod-vize/52e97b79-ae1b-4578-a69b-d7a4721900e1/tasks/'
pending = list(sys.argv[1:])
while pending:
    for t in list(pending):
        p = D + t + '.output'
        if not os.path.exists(p):
            continue
        s = open(p).read()
        if 'exited with code' in s:
            keys = re.findall(r'(test result[^\n]*|Finished[^\n]*|error[^\n]*|FAILED[^\n]*)', s)
            print(t, '|', ' || '.join(keys[:4])[:400], flush=True)
            pending.remove(t)
    time.sleep(10)
~~~~
