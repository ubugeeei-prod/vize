/**
 * How the type-check profile names what a perf sample spent: the process
 * role, the DSO group of a frame, the cost kind of a sample's leaf (libc
 * allocator, kernel syscall, Go GC, ...) and the phase a stack belongs to
 * (Vize generation, reply encoding, host decode, parse, bind, check, ...).
 *
 * Phases are ordered rules over the whole stack: the first rule all of whose
 * patterns match some frame wins. Frame names here are already cleaned
 * (`cleanSymbol`): Rust hashes and the tsgo module prefix removed.
 */

import { basename } from "node:path";

/** A lane's process tree: who is the root and who are the node children. */
export type LaneShape = "host" | "check";

const TSGO_MODULE = "github.com/microsoft/TypeScript/tsc/";

/** Strip Rust legacy hashes, the tsgo module path and frame separators. */
export function cleanSymbol(symbol: string): string {
  return symbol
    .replace(/::h[0-9a-f]{16}$/u, "")
    .replaceAll(TSGO_MODULE, "")
    .replaceAll(";", ":")
    .replaceAll("\n", " ");
}

/** Coarse library a DSO path belongs to. */
export function dsoGroup(dso: string): string {
  const base = basename(dso);
  if (dso.startsWith("[kernel") || /\.ko(?:\.\w+)?$/u.test(base)) return "kernel";
  if (dso === "[vdso]" || dso === "[vsyscall]") return "vdso";
  if (dso === "[unknown]" || dso === "") return "unknown";
  if (/^perf-\d+\.map$/u.test(base) || dso.startsWith("[JIT]") || base.startsWith("jitted-")) {
    return "node JIT";
  }
  if (/^vize-vitrine\..*\.node$/u.test(base)) return "vize native";
  if (/^lib(?:c|pthread|rt|dl)[.-]/u.test(base)) return "libc";
  if (/^ld-linux.*\.so/u.test(base)) return "ld.so";
  if (/^libm[.-]/u.test(base)) return "libm";
  if (base.startsWith("libstdc++")) return "libstdc++";
  if (base.startsWith("libgcc_s")) return "libgcc";
  if (base === "node") return "node";
  if (base === "tsrs") return "tsrs";
  if (base === "tsc" && /@typescript\/typescript-[^/]+\/lib\/tsc$/u.test(dso)) return "tsgo";
  return base;
}

/** The role of a process from the image it executed and whether it is the lane's root. */
export function processRole(image: string, root: boolean, shape: LaneShape): string {
  if (image === "tsc") return shape === "check" ? "tsgo (Corsa)" : "tsgo";
  if (image === "tsrs" || image.startsWith("tsrs")) return "tsrs";
  if (image === "node") {
    if (shape === "host") return root ? "host launcher (node)" : "vize mapper (node)";
    return root ? "vize check (node)" : "node (child)";
  }
  return image;
}

/** Leaf cost kinds; the first matching entry wins. `stack` is leaf-first. */
const KINDS: Array<{ kind: string; leaf?: RegExp; group?: string; stack?: RegExp }> = [
  { kind: "kernel: page fault", group: "kernel", stack: /^(?:asm_)?exc_page_fault$/u },
  { kind: "kernel: futex", group: "kernel", stack: /^__x64_sys_futex|^futex_/u },
  { kind: "kernel: read/write", group: "kernel", stack: /^(?:__x64_sys|ksys)_(?:p?read|p?write)/u },
  {
    kind: "kernel: mmap/munmap",
    group: "kernel",
    stack: /^__x64_sys_(?:mmap|munmap|mprotect|madvise|brk)/u,
  },
  { kind: "kernel: sched", group: "kernel", stack: /^(?:schedule|__schedule|finish_task_switch)/u },
  { kind: "kernel: other syscall", group: "kernel", stack: /^__x64_sys_|^do_syscall_64$/u },
  { kind: "kernel: other", group: "kernel" },
  {
    kind: "libc: malloc/free",
    group: "libc",
    leaf: /^(?:__libc_)?(?:malloc|free|cfree|realloc|calloc|memalign|posix_memalign|aligned_alloc)$|^_int_(?:malloc|free|realloc|memalign)|^malloc_|^tcache_|^unlink_chunk|^sysmalloc|^_int_free_|^__malloc|^arena_|^alloc_perturb/u,
  },
  { kind: "libc: memcpy/memmove", group: "libc", leaf: /mem(?:cpy|move)/u },
  {
    kind: "libc: memset/memcmp/str*",
    group: "libc",
    leaf: /^__(?:mem|str|wmem)|^(?:mem|str)\w+$/u,
  },
  {
    kind: "libc: pthread/futex",
    group: "libc",
    leaf: /pthread|futex|lll_lock|lll_unlock|__new_sem|sem_/u,
  },
  {
    kind: "libc: read/write",
    group: "libc",
    leaf: /^(?:__libc_)?(?:read|write|readv|writev|pread64|pwrite64)$/u,
  },
  { kind: "libc: other", group: "libc" },
  {
    kind: "Go: alloc",
    group: "tsgo",
    leaf: /^runtime\.(?:mallocgc|newobject|makeslice|growslice|makemap|newarray|nextFreeFast|\(\*mcache\)|\(\*mcentral\)|\(\*mheap\)|\(\*mspan\)\.(?:init|nextFreeIndex)|memclrNoHeapPointers|heapSetType|rawstring|rawbyteslice|concatstring|slicebytetostring|mallocgc\w*|deductAssistCredit)/u,
  },
  {
    kind: "Go: GC",
    group: "tsgo",
    leaf: /^runtime\.(?:gc|scanobject|scanblock|scanstack|scanframe|markroot|greyobject|findObject|wbBuf|bulkBarrierPreWrite|typePointers|\(\*gcWork\)|\(\*gcBits\)|\(\*mspan\)\.(?:sweep|typePointers|heapBits)|\(\*sweepLocked\)|sweepone|bgsweep|bgscavenge|\(\*pageAlloc\)|\(\*scavenger)|^internal\/runtime\/gc/u,
  },
  {
    kind: "Go: scheduler",
    group: "tsgo",
    leaf: /^runtime\.(?:schedule|findRunnable|park_m|mcall|futex|usleep|nanotime|notesleep|notewakeup|stealWork|runqgrab|netpoll|lock2|unlock2|osyield|procyield|semacquire|semrelease|chansend|chanrecv|selectgo|gopark|goready|ready|wakep|startm|stopm|systemstack|morestack|newstack|copystack)/u,
  },
  { kind: "Go: maps", group: "tsgo", leaf: /^(?:runtime\.map|internal\/runtime\/maps\.)/u },
  {
    kind: "Go: memmove/hash",
    group: "tsgo",
    leaf: /^(?:runtime\.(?:memmove|memequal|memhash|strhash|aeshash|cmpstring)|internal\/bytealg)/u,
  },
  { kind: "Go: runtime other", group: "tsgo", leaf: /^runtime\./u },
  {
    kind: "Rust: alloc",
    leaf: /^(?:__rust_(?:alloc|dealloc|realloc)|alloc::alloc::|<alloc::raw_vec|alloc::raw_vec::|mi_|_mi_)/u,
  },
  { kind: "dynamic linking", group: "ld.so" },
  { kind: "V8/node JIT", group: "node JIT" },
  { kind: "node/V8", group: "node" },
  { kind: "unresolved", group: "unknown" },
];

/** Cost kind of one sample from its leaf-first frames and their DSO groups. */
export function costKind(frames: string[], groups: string[]): string {
  const leaf = frames[0] ?? "";
  const leafGroup = groups[0] ?? "unknown";
  for (const rule of KINDS) {
    if (rule.group && rule.group !== leafGroup) continue;
    if (rule.leaf && !rule.leaf.test(leaf)) continue;
    if (rule.stack) {
      const pattern = rule.stack;
      // Only the kernel part of the chain decides a kernel kind.
      const kernel = frames.filter((_, index) => groups[index] === "kernel");
      if (!kernel.some((frame) => pattern.test(frame))) continue;
    }
    return rule.kind;
  }
  return "code";
}

type PhaseRule = { phase: string; all: RegExp[]; roles?: RegExp };

const GO = (pattern: string) => new RegExp(`^internal/${pattern}`, "u");
const JSON_GO = /^(?:encoding\/json|internal\/json)/u;

/** Ordered phase rules; frames are cleaned names, any frame may match a pattern. */
export const PHASE_RULES: PhaseRule[] = [
  // tsgo / Corsa: the content mapper client and the IPC under it.
  { phase: "host: mapper reply decode", all: [GO("contentmapper\\."), JSON_GO] },
  { phase: "host: IPC JSON", all: [GO("ipc\\."), JSON_GO] },
  {
    phase: "host: mapper span/directive apply",
    all: [
      /^internal\/contentmapper\.(?:decodeMappedOutput|normalize|newPositionNormalizer|\(\*positionNormalizer\))/u,
    ],
  },
  { phase: "host: parse mapped SFC", all: [GO("contentmapper\\."), GO("(?:parser|scanner)\\.")] },
  { phase: "host: IPC framing/io", all: [GO("ipc\\.")] },
  { phase: "host: mapper client other", all: [GO("contentmapper\\.")] },
  { phase: "host: API server", all: [GO("api\\.|^internal/api/")] },
  { phase: "host: parse", all: [GO("(?:parser|scanner|jsdoc)\\.")] },
  { phase: "host: bind", all: [GO("binder\\.")] },
  { phase: "host: check", all: [GO("checker\\.")] },
  {
    phase: "host: emit/diagnostics",
    all: [GO("(?:printer|diagnosticwriter|diagnostics|transformers|outputpaths)")],
  },
  {
    phase: "host: program/module resolution",
    all: [GO("(?:compiler|module|modulespecifiers|tspath|vfs|tsoptions|packagejson|execute)")],
  },
  {
    phase: "host: Go GC (background)",
    all: [/^runtime\.(?:gcBgMarkWorker|bgsweep|bgscavenge|gcDrain)/u],
  },
  // Vize: generation, the reply wire format, the request side and the writer.
  {
    phase: "vize: reply encode",
    all: [
      /content_mapper::dispatch::(?:wire|encode)|::wire::(?:encode|push_|finish_frame)|encode_transform|encode_frame/u,
    ],
  },
  {
    phase: "vize: generate (mapper)",
    all: [/generate_vue_content_mapper_transform|content_mapper::dispatch::transform_frame/u],
  },
  {
    phase: "vize: request decode",
    all: [
      /content_mapper::(?:read_frame|dispatch::(?:read_loop|dispatch_transform))|serde_json::de::/u,
    ],
  },
  {
    phase: "vize: reply write",
    all: [/content_mapper::dispatch::replies|std::io::Write::write_all|BufWriter/u],
  },
  { phase: "vize: Corsa client", all: [/corsa|typecheck_service/iu] },
  { phase: "vize: Canon", all: [/^(?:<)?vize_canon::/u] },
  {
    phase: "vize: SFC compile/parse",
    all: [/^(?:<)?vize_(?:atelier|armature|croquis|relief|glyph|l\d)\w*::/u],
  },
  { phase: "vize: other", all: [/^(?:<)?vize\w*::/u] },
  { phase: "node: addon load", all: [/^(?:dlopen|_dl_|dl_open_worker)/u] },
];

/** First phase rule matching the stack; samples of unmatched stacks fall to their role. */
export function phaseOf(frames: string[], role: string): string {
  for (const rule of PHASE_RULES) {
    if (rule.roles && !rule.roles.test(role)) continue;
    if (rule.all.every((pattern) => frames.some((frame) => pattern.test(frame)))) return rule.phase;
  }
  return `${role}: other`;
}

/** Outermost frames that mean the unwinder reached a thread or process entry. */
const ENTRY =
  /^(?:_start|__libc_start_main\w*|__libc_start_call_main|start_thread|clone3?|__clone3?|__GI___clone3?|runtime\.(?:goexit|mstart|rt0_go)\w*(?:\.abi0)?|_rt0_amd64\w*|std::sys::.*thread_start|std::rt::lang_start\w*|main|uv__thread_start|node::Start)$/u;

export const isEntryFrame = (frame: string): boolean => ENTRY.test(frame);
