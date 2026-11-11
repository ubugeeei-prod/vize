/**
 * One pass over `perf script` text of a whole lane process tree:
 *
 *   perf script -F comm,pid,tid,period,event,ip,sym,dso \
 *     --show-task-events --show-mmap-events --no-inline
 *
 * Task events give each pid the image it executed (so the node launcher that
 * execve()s tsgo is told apart from tsgo) and fork-inherited mappings; mmap
 * events place frames of stripped hosts in their ELF, named through
 * typecheck-profile-elf. Each sample lands in per-role, per-DSO, per-function
 * (self and inclusive), phase x cost-kind and folded-stack tables.
 */

import { type FunctionTable, functionAt, functionTable, vaddrOf } from "./typecheck-profile-elf.ts";
import {
  type LaneShape,
  PHASE_RULES,
  cleanSymbol,
  costKind,
  dsoGroup,
  isEntryFrame,
  processRole,
} from "./typecheck-profile-rules.ts";

export type FunctionCost = { role: string; dso: string; name: string; self: number; total: number };

export type Profile = {
  event: string;
  samples: number;
  period: number;
  roles: Record<string, number>;
  dsos: Record<string, number>;
  roleDsos: Record<string, Record<string, number>>;
  /** role -> phase -> cost kind -> period */
  phases: Record<string, Record<string, Record<string, number>>>;
  kinds: Record<string, number>;
  /** Inclusive period under each `__x64_sys_*` entry. */
  syscalls: Record<string, number>;
  functions: FunctionCost[];
  quality: Record<string, { samples: number; complete: number; roots: Array<[string, number]> }>;
  unresolved: Record<string, number>;
  symbolizers: Record<string, FunctionTable["kind"]>;
};

type Mapping = { start: number; end: number; pgoff: number; path: string };
type Frame = { ip: number; sym: string; dso: string };
type Sample = { comm: string; pid: number; period: number; event: string; frames: Frame[] };

const SAMPLE = /^(.+?)\s+(-?\d+)\/(-?\d+)\s+(?:\[\d+\]\s+)?(?:\d+\.\d+:\s+)?(\d+)\s+(\S+?):?\s*$/u;
const FRAME = /^\s+([0-9a-f]+)\s+(.*?)\s*\(([^()]*)\)$/u;
const EXEC = /PERF_RECORD_COMM exec: (.+):(-?\d+)\/(-?\d+)$/u;
const FORK = /PERF_RECORD_FORK\((-?\d+):(-?\d+)\):\((-?\d+):(-?\d+)\)/u;
const MMAP =
  /PERF_RECORD_MMAP2? (-?\d+)\/(-?\d+): \[0x([0-9a-f]+)\(0x([0-9a-f]+)\) @ (0x[0-9a-f]+|\d+)[^\]]*\]: (\S+) (.+)$/u;
const SYSCALL = /^__(?:x64|ia32|se|do)_sys_(\w+)$/u;
const SYMBOLIZED_GROUPS = new Set(["tsgo", "tsrs"]);

const add = (record: Record<string, number>, key: string, value: number) => {
  record[key] = (record[key] ?? 0) + value;
};

/** Pattern bitmask per frame name, so a phase rule tests each name once. */
class PhaseMatcher {
  private readonly patterns = PHASE_RULES.flatMap((rule) => rule.all);
  private readonly words = Math.ceil(this.patterns.length / 32);
  private readonly cache = new Map<string, Uint32Array>();
  private readonly ruleMasks = PHASE_RULES.map((rule) => {
    const mask = new Uint32Array(this.words);
    for (const pattern of rule.all) {
      const index = this.patterns.indexOf(pattern);
      mask[index >> 5]! |= 1 << (index & 31);
    }
    return mask;
  });

  private mask(name: string): Uint32Array {
    let mask = this.cache.get(name);
    if (!mask) {
      mask = new Uint32Array(this.words);
      this.patterns.forEach((pattern, index) => {
        if (pattern.test(name)) mask![index >> 5]! |= 1 << (index & 31);
      });
      this.cache.set(name, mask);
    }
    return mask;
  }

  phase(names: string[], role: string): string {
    const stack = new Uint32Array(this.words);
    for (const name of names) {
      const mask = this.mask(name);
      for (let word = 0; word < this.words; word++) stack[word]! |= mask[word]!;
    }
    for (const [index, rule] of PHASE_RULES.entries()) {
      if (rule.roles && !rule.roles.test(role)) continue;
      const need = this.ruleMasks[index]!;
      let ok = true;
      for (let word = 0; word < this.words && ok; word++) {
        ok = (stack[word]! & need[word]!) === need[word]!;
      }
      if (ok) return rule.phase;
    }
    return `${role}: other`;
  }
}

export class ProfileFolder {
  private readonly images = new Map<number, string>();
  private readonly maps = new Map<number, Mapping[]>();
  private readonly tables = new Map<string, FunctionTable | null>();
  private readonly frameIds = new Map<string, number>();
  private readonly frameNames: string[] = [];
  private readonly stacks = new Map<string, number>();
  private readonly functions = new Map<string, FunctionCost>();
  private readonly kindCache = new Map<string, string>();
  private readonly phases = new PhaseMatcher();
  private readonly roots = new Map<string, Map<string, number>>();
  private root: number | null = null;
  private current: Sample | null = null;
  private readonly profile: Profile = {
    event: "",
    samples: 0,
    period: 0,
    roles: {},
    dsos: {},
    roleDsos: {},
    phases: {},
    kinds: {},
    syscalls: {},
    functions: [],
    quality: {},
    unresolved: {},
    symbolizers: {},
  };

  private readonly shape: LaneShape;
  private readonly loadTable: (path: string) => FunctionTable;

  constructor(shape: LaneShape, loadTable: (path: string) => FunctionTable = functionTable) {
    this.shape = shape;
    this.loadTable = loadTable;
  }

  line(text: string): void {
    if (text.trim() === "") {
      this.flush();
      return;
    }
    if (text.includes("PERF_RECORD_")) {
      this.flush();
      this.sideBand(text);
      return;
    }
    // Callchain lines are indented; a header is never, unless perf pads its comm.
    const frame = this.current && /^\s/u.test(text) ? FRAME.exec(text) : null;
    if (frame) {
      this.current!.frames.push({
        ip: Number.parseInt(frame[1]!, 16),
        sym: frame[2]!,
        dso: frame[3]!,
      });
      return;
    }
    const match = SAMPLE.exec(text);
    if (!match) return;
    this.flush();
    this.current = {
      comm: match[1]!.trim(),
      pid: Number(match[2]),
      period: Number(match[4]),
      event: match[5]!,
      frames: [],
    };
  }

  private sideBand(text: string): void {
    const exec = EXEC.exec(text);
    if (exec) {
      const pid = Number(exec[2]);
      this.images.set(pid, exec[1]!);
      this.maps.set(pid, []);
      this.root ??= pid;
      return;
    }
    const fork = FORK.exec(text);
    if (fork) {
      const [child, parent] = [Number(fork[1]), Number(fork[3])];
      if (child !== parent) {
        const image = this.images.get(parent);
        if (image) this.images.set(child, image);
        this.maps.set(child, [...(this.maps.get(parent) ?? [])]);
      }
      return;
    }
    const mmap = MMAP.exec(text);
    if (mmap) {
      const pid = Number(mmap[1]);
      const start = Number.parseInt(mmap[3]!, 16);
      const mapping = {
        start,
        end: start + Number.parseInt(mmap[4]!, 16),
        pgoff: Number(mmap[5]),
        path: mmap[7]!.trim(),
      };
      if (!this.maps.has(pid)) this.maps.set(pid, []);
      this.maps.get(pid)!.push(mapping);
      return;
    }
  }

  private table(path: string): FunctionTable | null {
    if (!this.tables.has(path)) {
      let table: FunctionTable | null = null;
      try {
        table = this.loadTable(path);
      } catch {
        table = null;
      }
      this.tables.set(path, table);
      this.profile.symbolizers[path] = table?.kind ?? "none";
    }
    return this.tables.get(path) ?? null;
  }

  /** Name a frame perf could not, from the stripped binary's own tables. */
  private symbolize(pid: number, frame: Frame, group: string): string {
    if (frame.sym !== "[unknown]" || !SYMBOLIZED_GROUPS.has(group)) return frame.sym;
    const table = this.table(frame.dso);
    if (!table || table.kind === "symtab" || table.kind === "none") return frame.sym;
    const mapping = this.maps
      .get(pid)
      ?.find((map) => map.path === frame.dso && frame.ip >= map.start && frame.ip < map.end);
    let vaddr: number | null = null;
    if (mapping) vaddr = vaddrOf(table, frame.ip - mapping.start + mapping.pgoff);
    // A fixed-address executable needs no mapping: its runtime address is the vaddr.
    else if ((table.loads[0]?.vaddr ?? 0) > 0) vaddr = frame.ip;
    return (vaddr !== null && functionAt(table, vaddr)) || frame.sym;
  }

  private frameId(name: string): number {
    let id = this.frameIds.get(name);
    if (id === undefined) {
      id = this.frameNames.length;
      this.frameIds.set(name, id);
      this.frameNames.push(name);
    }
    return id;
  }

  private kind(names: string[], groups: string[]): string {
    const leafGroup = groups[0] ?? "unknown";
    const kernel =
      leafGroup === "kernel" ? names.filter((_, i) => groups[i] === "kernel").join("\n") : "";
    const key = `${leafGroup}\0${names[0] ?? ""}\0${kernel}`;
    let kind = this.kindCache.get(key);
    if (kind === undefined) {
      kind = costKind(names, groups);
      this.kindCache.set(key, kind);
    }
    return kind;
  }

  private flush(): void {
    const sample = this.current;
    this.current = null;
    if (!sample) return;
    const p = this.profile;
    const period = sample.period > 0 ? sample.period : 1;
    p.event ||= sample.event;
    p.samples++;
    p.period += period;
    this.root ??= sample.pid;
    const image = this.images.get(sample.pid) ?? sample.comm;
    const role = processRole(image, sample.pid === this.root, this.shape);
    const groups = sample.frames.map((frame) => dsoGroup(frame.dso));
    const names = sample.frames.map((frame, index) => {
      const symbol = this.symbolize(sample.pid, frame, groups[index]!);
      if (symbol === "[unknown]") {
        add(p.unresolved, groups[index]!, period);
        return `[unknown ${groups[index]}]`;
      }
      return cleanSymbol(symbol);
    });
    const leafGroup = groups[0] ?? "unknown";
    add(p.roles, role, period);
    add(p.dsos, leafGroup, period);
    add((p.roleDsos[role] ??= {}), leafGroup, period);
    const kind = this.kind(names, groups);
    add(p.kinds, kind, period);
    const phase = this.phases.phase(names, role);
    add(((p.phases[role] ??= {})[phase] ??= {}), kind, period);

    const seen = new Set<string>();
    const syscalls = new Set<string>();
    names.forEach((name, index) => {
      const key = `${role}\t${groups[index]}\t${name}`;
      let cost = this.functions.get(key);
      if (!cost) {
        cost = { role, dso: groups[index]!, name, self: 0, total: 0 };
        this.functions.set(key, cost);
      }
      if (index === 0) cost.self += period;
      if (!seen.has(key)) {
        seen.add(key);
        cost.total += period;
      }
      const syscall = groups[index] === "kernel" ? SYSCALL.exec(name) : null;
      if (syscall) syscalls.add(syscall[1]!);
    });
    for (const syscall of syscalls) add(p.syscalls, syscall, period);

    const quality = (p.quality[role] ??= { samples: 0, complete: 0, roots: [] });
    quality.samples += period;
    const outermost = names.at(-1) ?? "";
    if (isEntryFrame(outermost)) quality.complete += period;
    else {
      const roots = this.roots.get(role) ?? new Map<string, number>();
      this.roots.set(role, roots);
      roots.set(outermost, (roots.get(outermost) ?? 0) + period);
    }

    const ids = [this.frameId(role)];
    for (let index = names.length - 1; index >= 0; index--) {
      const name = names[index]!;
      ids.push(this.frameId(groups[index] === "kernel" ? `${name}_[k]` : name));
    }
    const stack = ids.join(",");
    this.stacks.set(stack, (this.stacks.get(stack) ?? 0) + period);
  }

  /** The aggregate tables; functions below `minShare` of the period in both self and total are dropped. */
  finish(minShare = 0.00005): Profile {
    this.flush();
    const floor = this.profile.period * minShare;
    this.profile.functions = [...this.functions.values()]
      .filter((cost) => cost.self >= floor || cost.total >= floor)
      .sort((a, b) => b.self - a.self || b.total - a.total);
    for (const [role, roots] of this.roots) {
      const quality = this.profile.quality[role];
      if (quality) quality.roots = [...roots].sort((a, b) => b[1] - a[1]).slice(0, 8);
    }
    return this.profile;
  }

  /** Folded stacks (`role;root;...;leaf period`), one per line, largest first. */
  *folded(): Generator<string> {
    const entries = [...this.stacks].sort((a, b) => b[1] - a[1]);
    for (const [stack, period] of entries) {
      const names = stack.split(",").map((id) => this.frameNames[Number(id)]!);
      yield `${names.join(";")} ${Math.round(period)}`;
    }
  }
}
