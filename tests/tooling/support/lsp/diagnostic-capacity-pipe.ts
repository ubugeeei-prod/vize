import type { LspSession } from "./session.ts";
import { waitUntil } from "./diagnostic-capacity.ts";
import type { NativeDiagnosticsGate } from "./responsive-lint-native-gate.ts";
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import type { NativeLife } from "./responsive-lint-native-gate.ts";

export type PipeEvidence = {
  pid: number;
  birthTicks: string;
  executable: string;
  queuedBytes: number | null;
  available: boolean;
  error?: string;
};

/** FIONREAD observes a paused child's stdin pipe without consuming native wire bytes. */
export function nativePipeEvidence(lives: NativeLife[]): PipeEvidence[] {
  const code = [
    "import sys,json,os,fcntl,termios,array",
    "rows=[]",
    "for expected in json.loads(sys.argv[1]):",
    " pid=expected['pid'];raw=open('/proc/'+str(pid)+'/stat').read();fields=raw[raw.rfind(') ')+2:].split()",
    " assert fields[19]==expected['birthTicks'] and fields[0]=='T'",
    " assert os.path.realpath('/proc/'+str(pid)+'/exe')==expected['executable']",
    " row={'pid':pid,'birthTicks':fields[19],'executable':expected['executable'],'queuedBytes':None,'available':False}",
    " try:",
    "  fd=os.open('/proc/'+str(pid)+'/fd/0',os.O_RDONLY|os.O_NONBLOCK|os.O_CLOEXEC)",
    "  try:",
    "   value=array.array('i',[0]);fcntl.ioctl(fd,termios.FIONREAD,value,True);row.update(queuedBytes=value[0],available=True)",
    "  finally: os.close(fd)",
    " except OSError as error: row['error']=str(error)",
    " rows.append(row)",
    "print(json.dumps(rows))",
  ].join("\n");
  const parsed: unknown = JSON.parse(
    execFileSync("python3", ["-c", code, JSON.stringify(lives)], {
      encoding: "utf8",
      timeout: 10_000,
    }),
  );
  assert.ok(Array.isArray(parsed));
  return parsed as PipeEvidence[];
}
export function nativePipeGrew(before: PipeEvidence[], after: PipeEvidence[]): boolean {
  return after.some(
    (row) =>
      row.available &&
      before.some(
        (old) =>
          old.available &&
          old.pid === row.pid &&
          old.birthTicks === row.birthTicks &&
          old.queuedBytes !== null &&
          row.queuedBytes !== null &&
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
