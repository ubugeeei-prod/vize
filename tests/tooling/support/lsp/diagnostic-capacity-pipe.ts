import type { LspSession } from "./session.ts";
import { waitUntil } from "./diagnostic-capacity.ts";
import type { NativeDiagnosticsGate } from "./responsive-lint-native-gate.ts";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import type { NativeLife } from "./responsive-lint-native-gate.ts";

export type SocketDescriptor = {
  pid: number;
  fd: number;
  inode: string;
  birthTicks: string;
  executable: string;
};
export type SocketEvidence = SocketDescriptor & {
  queuedBytes: number;
  localPath: string;
  filesystemInode: string | null;
  raw: string;
};
export type IpcEvidence = {
  source: "Linux AF_UNIX SOCK_DIAG";
  raw: string[];
  processes: NativeLife[];
  descriptors: SocketDescriptor[];
  channels: SocketEvidence[];
};

/** Join all three kernel identities; LISTEN queues count connections, not request bytes. */
export function decodeNativeSockets(
  raw: string[],
  descriptors: SocketDescriptor[],
): SocketEvidence[] {
  const result: SocketEvidence[] = [];
  for (const line of raw) {
    const fields = line.trim().split(/\s+/u);
    const state = fields.indexOf("ESTAB");
    if (state < 0) continue;
    const localPath = fields[state + 3];
    const queue = fields[state + 1];
    // The address column is the kernel socket inode; extended ino: identifies its filesystem path.
    const inode = fields[state + 4];
    const filesystemInode = /\bino:(\d+)/u.exec(line)?.[1] ?? null;
    if (!localPath?.startsWith("/tmp/tsgo-api-") || !queue || !/^\d+$/u.test(queue)) continue;
    for (const match of line.matchAll(/\bpid=(\d+),fd=(\d+)/gu)) {
      const descriptor = descriptors.find(
        (row) => row.pid === Number(match[1]) && row.fd === Number(match[2]) && row.inode === inode,
      );
      if (descriptor)
        result.push({
          ...descriptor,
          queuedBytes: Number(queue),
          localPath,
          filesystemInode,
          raw: line,
        });
    }
  }
  return result;
}

/** Read kernel socket queue lengths without opening, peeking at, or consuming the RPC stream. */
export function nativeIpcEvidence(lives: NativeLife[]): IpcEvidence {
  const code = [
    "import sys,json,os,re,subprocess",
    "lives=json.loads(sys.argv[1]);descriptors=[];processes=[]",
    "for expected in lives:",
    " pid=expected['pid'];raw=open('/proc/'+str(pid)+'/stat').read();fields=raw[raw.rfind(') ')+2:].split()",
    " assert fields[19]==expected['birthTicks'] and fields[0]=='T'",
    " assert os.path.realpath('/proc/'+str(pid)+'/exe')==expected['executable']",
    " processes.append(dict(expected,state=fields[0],rawStat=raw))",
    " for name in os.listdir('/proc/'+str(pid)+'/fd'):",
    "  try: link=os.readlink('/proc/'+str(pid)+'/fd/'+name)",
    "  except FileNotFoundError: continue",
    "  match=re.fullmatch(r'socket:\\[(\\d+)\\]',link)",
    "  if match: descriptors.append({'pid':pid,'fd':int(name),'inode':match[1],'birthTicks':expected['birthTicks'],'executable':expected['executable']})",
    "output=subprocess.run(['ss','--unix','--all','--numeric','--processes','--extended','--no-header'],capture_output=True,text=True,check=True).stdout",
    "owned={row['pid'] for row in lives}",
    "lines=[line for line in output.splitlines() if any(int(pid) in owned for pid in re.findall(r'pid=(\\d+),fd=',line))]",
    "print(json.dumps({'raw':lines,'descriptors':descriptors,'processes':processes}))",
  ].join("\n");
  const parsed: unknown = JSON.parse(
    execFileSync("python3", ["-c", code, JSON.stringify(lives)], {
      encoding: "utf8",
      timeout: 10_000,
    }),
  );
  assert.ok(
    parsed &&
      typeof parsed === "object" &&
      "raw" in parsed &&
      "descriptors" in parsed &&
      "processes" in parsed,
  );
  assert.ok(
    Array.isArray(parsed.raw) &&
      Array.isArray(parsed.descriptors) &&
      Array.isArray(parsed.processes),
  );
  const raw = parsed.raw as string[],
    descriptors = parsed.descriptors as SocketDescriptor[];
  return {
    source: "Linux AF_UNIX SOCK_DIAG",
    raw,
    processes: parsed.processes as NativeLife[],
    descriptors,
    channels: decodeNativeSockets(raw, descriptors),
  };
}
export function nativeIpcGrew(before: IpcEvidence, after: IpcEvidence): boolean {
  return after.channels.some((row) =>
    before.channels.some(
      (old) =>
        old.pid === row.pid &&
        old.birthTicks === row.birthTicks &&
        old.fd === row.fd &&
        old.inode === row.inode &&
        old.executable === row.executable &&
        old.localPath === row.localPath &&
        row.queuedBytes > old.queuedBytes,
    ),
  );
}

/** Retain identities before the parent exits; later orphan cleanup never signals a reused PID. */
export function ownedProcessSnapshot(owner: number): NativeLife[] {
  const code = [
    "import os,json,sys",
    "all={}",
    "for name in os.listdir('/proc'):",
    " if not name.isdecimal(): continue",
    " try:",
    "  raw=open('/proc/'+name+'/stat').read();f=raw[raw.rfind(') ')+2:].split()",
    "  all[int(name)]={'pid':int(name),'parent':int(f[1]),'birthTicks':f[19],'state':f[0],'executable':os.path.realpath('/proc/'+name+'/exe'),'rawStat':raw}",
    " except OSError: pass",
    "root=int(sys.argv[1]);owned=[]",
    "for pid,row in all.items():",
    " seen=set();current=pid",
    " while current in all and current not in seen:",
    "  if current==root: owned.append(row);break",
    "  seen.add(current);current=all[current]['parent']",
    "print(json.dumps(owned))",
  ].join("\n");
  return JSON.parse(
    execFileSync("python3", ["-c", code, String(owner)], {
      encoding: "utf8",
      timeout: 10_000,
    }),
  ) as NativeLife[];
}

export function signalBoundProcesses(
  lives: NativeLife[],
  action: "SIGCONT" | "SIGTERM" | "SIGKILL" | "observe",
) {
  const code = [
    "import os,json,sys,signal",
    "rows=[]",
    "for expected in json.loads(sys.argv[1]):",
    " row={'pid':expected['pid'],'birthTicks':expected['birthTicks'],'alive':False,'signal':sys.argv[2],'sent':False}",
    " fd=None",
    " try:",
    "  fd=os.pidfd_open(expected['pid'],0)",
    "  raw=open('/proc/'+str(expected['pid'])+'/stat').read();f=raw[raw.rfind(') ')+2:].split()",
    "  match=f[19]==expected['birthTicks'] and os.path.realpath('/proc/'+str(expected['pid'])+'/exe')==expected['executable']",
    "  row.update(matched=match,pidfdBound=True,state=f[0],alive=match and f[0]!='Z')",
    "  if match and f[0]!='Z' and sys.argv[2]!='observe': signal.pidfd_send_signal(fd,getattr(signal,sys.argv[2]));row['sent']=True",
    " except ProcessLookupError: pass",
    " except FileNotFoundError: pass",
    " except OSError as error: row['error']=str(error)",
    " finally:",
    "  if fd is not None: os.close(fd)",
    " rows.append(row)",
    "print(json.dumps(rows))",
  ].join("\n");
  return JSON.parse(
    execFileSync("python3", ["-c", code, JSON.stringify(lives), action], {
      encoding: "utf8",
      timeout: 10_000,
    }),
  ) as { pid: number; birthTicks: string; alive: boolean; matched?: boolean; error?: string }[];
}

export async function cleanupCapacitySession(session: LspSession, gate?: NativeDiagnosticsGate) {
  const report: Record<string, unknown> = {};
  const errors: string[] = [];
  let saved = gate?.lives ?? [];
  try {
    saved = [...ownedProcessSnapshot(session.processId), ...saved];
  } catch (error) {
    errors.push(String(error));
  }
  try {
    report.resume = signalBoundProcesses(saved, "SIGCONT");
  } catch (error) {
    errors.push(String(error));
  }
  try {
    gate?.resume();
  } catch (error) {
    errors.push(String(error));
  }
  try {
    await session.shutdown();
  } catch (error) {
    errors.push(String(error));
  }
  let complete = false;
  try {
    const live = () => signalBoundProcesses(saved, "observe").some((row) => row.alive || row.error);
    if (live()) {
      report.term = signalBoundProcesses(saved, "SIGTERM");
      if (!(await waitUntil(() => !live(), 2_000)))
        report.kill = signalBoundProcesses(saved, "SIGKILL");
    }
    complete = await waitUntil(() => !live(), 2_000);
  } catch (error) {
    errors.push(String(error));
    try {
      await session.kill();
    } catch (killError) {
      errors.push(String(killError));
    }
  }
  try {
    await session.waitForClosed();
  } catch (error) {
    complete = false;
    errors.push(String(error));
  }
  return { complete, errors, saved, ...report };
}
