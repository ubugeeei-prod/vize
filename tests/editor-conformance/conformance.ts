// TS-45 judge: evaluates one editor's transcript (written by `lsp-tap.mjs`)
// against the single scenario in `scenario.json`. Every client is judged by
// the same code on the same expectations — there is no per-client branch
// anywhere in this file, and every comparison is exact deep equality after
// URI normalization.
import { isDeepStrictEqual } from "node:util";

import { applyContentChange, type ContentChange } from "./support/document.ts";

export type Entry = {
  session: number;
  /** Milliseconds since the tap started; `null` on the exit recorder's line. */
  t: number | null;
  dir?: "c2s" | "s2c";
  msg?: Message;
  event?: string;
  code?: number | null;
  error?: string;
};
type Message = { id?: number | string; method?: string; params?: any; result?: any; error?: any };

export type Step =
  | { id: string; label: string; kind: "initialize"; expect: unknown }
  | { id: string; label: string; kind: "open"; expect: string }
  | { id: string; label: string; kind: "diagnostics"; expect: unknown[] }
  | { id: string; label: string; kind: "request"; method: string; match: object; expect: unknown }
  | { id: string; label: string; kind: "text"; action: object; expect: string }
  | { id: string; label: string; kind: "shutdown"; expect: { exitCode: number } };

export type Scenario = {
  suite: string;
  document: string;
  languageId: string;
  initializationOptions: object;
  steps: Step[];
};

export type StepResult = { id: string; label: string; pass: boolean; detail: string };
export type Evaluation = { session: number | null; messages: number; steps: StepResult[] };

export const WORKSPACE = "${workspace}";

/** Rewrites every `file://` URI under a workspace root (values and keys) to `${workspace}/…`. */
export function normalizeUris(value: unknown, roots: readonly string[]): unknown {
  const uri = (text: string): string => {
    if (!text.startsWith("file://")) return text;
    const path = decodeURIComponent(text.slice("file://".length));
    for (const root of roots) {
      if (path === root || path.startsWith(`${root}/`)) return WORKSPACE + path.slice(root.length);
    }
    return text;
  };
  if (typeof value === "string") return uri(value);
  if (Array.isArray(value)) return value.map((item) => normalizeUris(item, roots));
  if (value !== null && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [uri(key), normalizeUris(item, roots)]),
    );
  }
  return value;
}

function compareDiagnostics(left: any, right: any): number {
  const keys = (d: any) => [
    d.range.start.line,
    d.range.start.character,
    d.range.end.line,
    d.range.end.character,
    d.severity ?? 0,
    String(d.source ?? ""),
    String(d.code ?? ""),
    String(d.message),
  ];
  const [a, b] = [keys(left), keys(right)];
  for (let index = 0; index < a.length; index += 1) {
    if (a[index] < b[index]) return -1;
    if (a[index] > b[index]) return 1;
  }
  return 0;
}

/** Diagnostic order inside one publish carries no meaning; everything else is compared as sent. */
export function canonicalDiagnostics(diagnostics: unknown[]): unknown[] {
  return [...diagnostics].sort(compareDiagnostics);
}

function mismatch(label: string, expected: unknown, actual: unknown): string {
  return `${label}\nexpected: ${JSON.stringify(expected)}\nactual:   ${JSON.stringify(actual)}`;
}

function matches(actual: any, pattern: any): boolean {
  if (pattern === null || typeof pattern !== "object" || Array.isArray(pattern)) {
    return isDeepStrictEqual(actual, pattern);
  }
  return Object.entries(pattern).every(([key, value]) => matches(actual?.[key], value));
}

/**
 * `texts[i]` is the document after message `i`; `states[i]` also lists every
 * intermediate state inside one didChange, because a client may batch several
 * edits (Neovim debounces a formatting edit and the next keystroke into one
 * notification) and the server applies them in order.
 */
type Timeline = {
  messages: Message[];
  directions: Array<"c2s" | "s2c">;
  texts: Array<string | null>;
  states: string[][];
};

function timeline(entries: Entry[], session: number, uri: string, roots: string[]): Timeline {
  const messages: Message[] = [];
  const directions: Array<"c2s" | "s2c"> = [];
  const texts: Array<string | null> = [];
  const states: string[][] = [];
  let text: string | null = null;
  for (const entry of entries) {
    if (entry.session !== session || entry.msg == null || entry.dir == null) continue;
    const message = normalizeUris(entry.msg, roots) as Message;
    const seen: string[] = [];
    if (entry.dir === "c2s" && message.params?.textDocument?.uri === uri) {
      if (message.method === "textDocument/didOpen") text = message.params.textDocument.text;
      if (message.method === "textDocument/didChange" && text != null) {
        for (const change of message.params.contentChanges as ContentChange[]) {
          text = applyContentChange(text, change);
          seen.push(text);
        }
      }
      if (message.method === "textDocument/didClose") text = null;
    }
    messages.push(message);
    directions.push(entry.dir);
    texts.push(text);
    states.push(seen);
  }
  return { messages, directions, texts, states };
}

function responseTo(line: Timeline, index: number): Message | undefined {
  const id = line.messages[index].id;
  return line.messages.find(
    (message, at) =>
      at > index && line.directions[at] === "s2c" && message.method == null && message.id === id,
  );
}

function isMutation(line: Timeline, index: number, uri: string): boolean {
  const message = line.messages[index];
  return (
    line.directions[index] === "c2s" &&
    (message.method === "textDocument/didChange" || message.method === "textDocument/didClose") &&
    message.params?.textDocument?.uri === uri
  );
}

/** Judges one transcript. `roots` are the workspace paths as the client may spell them. */
export function evaluate(scenario: Scenario, entries: Entry[], roots: string[]): Evaluation {
  const uri = `${WORKSPACE}/${scenario.document}`;
  const openers = [
    ...new Set(
      entries
        .filter(
          (entry) =>
            entry.dir === "c2s" &&
            entry.msg?.method === "textDocument/didOpen" &&
            (normalizeUris(entry.msg.params?.textDocument?.uri, roots) as string) === uri,
        )
        .map((entry) => entry.session),
    ),
  ];
  if (openers.length !== 1) {
    const detail = `expected exactly one server session to open ${uri}, found ${openers.length} (tap sessions: ${openers.join(", ")})`;
    return {
      session: null,
      messages: 0,
      steps: scenario.steps.map(({ id, label }) => ({ id, label, pass: false, detail })),
    };
  }
  const session = openers[0];
  const line = timeline(entries, session, uri, roots);
  const exit = entries.find((entry) => entry.session === session && entry.event === "exit");
  const results: StepResult[] = [];
  // The judge walks the transcript once, in scenario order: `cursor` is the
  // message the previous client action matched, `floor` the intermediate
  // document state inside that message (text steps only).
  let cursor = -1;
  let floor = -1;
  let text: string | null = null;
  const fromClient = (i: number, method: string) =>
    line.directions[i] === "c2s" && line.messages[i].method === method;

  const judge = (step: Step): { pass: boolean; detail: string; at?: number; state?: number } => {
    switch (step.kind) {
      case "initialize": {
        const at = line.messages.findIndex((_, i) => fromClient(i, "initialize"));
        if (at < 0) return { pass: false, detail: "the client never sent initialize" };
        const options = line.messages[at].params?.initializationOptions;
        if (!isDeepStrictEqual(options, scenario.initializationOptions)) {
          const detail = mismatch("initializationOptions", scenario.initializationOptions, options);
          return { pass: false, detail };
        }
        const result = responseTo(line, at)?.result;
        const actual = { capabilities: result?.capabilities, serverName: result?.serverInfo?.name };
        return isDeepStrictEqual(actual, step.expect)
          ? { pass: true, detail: "", at }
          : { pass: false, detail: mismatch("initialize result", step.expect, actual) };
      }
      case "open": {
        const at = line.messages.findIndex(
          (m, i) =>
            i > cursor &&
            fromClient(i, "textDocument/didOpen") &&
            m.params.textDocument.uri === uri,
        );
        if (at < 0) return { pass: false, detail: "the client never opened the scenario document" };
        const opened = line.messages[at].params.textDocument;
        const actual = { languageId: opened.languageId, text: opened.text };
        const expected = { languageId: scenario.languageId, text: step.expect };
        if (!isDeepStrictEqual(actual, expected)) {
          return { pass: false, detail: mismatch("didOpen", expected, actual) };
        }
        text = step.expect;
        return { pass: true, detail: "", at };
      }
      case "diagnostics": {
        let end = line.messages.findIndex((_, i) => i > cursor && isMutation(line, i, uri));
        if (end < 0) end = line.messages.length;
        let version: number | undefined;
        for (let i = end - 1; i >= 0 && version == null; i -= 1) {
          const document = line.messages[i].params?.textDocument;
          if (line.directions[i] === "c2s" && document?.uri === uri) version = document.version;
        }
        const published = line.messages.filter(
          (m, i) =>
            i > cursor &&
            i < end &&
            m.method === "textDocument/publishDiagnostics" &&
            m.params.uri === uri,
        );
        const last = published.at(-1)?.params;
        if (last == null) {
          return { pass: false, detail: "no diagnostics were published for this document state" };
        }
        if (last.version != null && last.version !== version) {
          const detail = `the settled publish carries version ${last.version}, the document is at ${version}`;
          return { pass: false, detail };
        }
        const actual = canonicalDiagnostics(last.diagnostics);
        return isDeepStrictEqual(actual, step.expect)
          ? { pass: true, detail: "" }
          : { pass: false, detail: mismatch("settled diagnostics", step.expect, actual) };
      }
      case "request": {
        const at = line.messages.findIndex(
          (m, i) =>
            i > cursor &&
            fromClient(i, step.method) &&
            m.id != null &&
            m.params?.textDocument?.uri === uri &&
            matches(m.params, step.match),
        );
        if (at < 0) {
          const detail = `the client never sent ${step.method} matching ${JSON.stringify(step.match)}`;
          return { pass: false, detail };
        }
        if (line.texts[at] !== text) {
          return {
            pass: false,
            detail: mismatch("document at request time", text, line.texts[at]),
          };
        }
        const response = responseTo(line, at);
        if (response == null) return { pass: false, detail: "the server never answered" };
        if (response.error != null) {
          const detail = `the server answered with an error: ${JSON.stringify(response.error)}`;
          return { pass: false, detail };
        }
        return isDeepStrictEqual(response.result ?? null, step.expect)
          ? { pass: true, detail: "", at }
          : { pass: false, detail: mismatch(step.method, step.expect, response.result) };
      }
      case "text": {
        const previous = text;
        // Later requests are judged against the scenario's intended document,
        // so a missed edit also fails the requests that depended on it.
        text = step.expect;
        for (let i = Math.max(cursor, 0); i < line.states.length; i += 1) {
          const state = line.states[i].indexOf(step.expect, i === cursor ? floor + 1 : 0);
          if (state >= 0) return { pass: true, detail: "", at: i, state };
        }
        const reached = line.texts.filter((_, i) => i > cursor && isMutation(line, i, uri)).at(-1);
        return { pass: false, detail: mismatch("document text", step.expect, reached ?? previous) };
      }
      case "shutdown": {
        const at = line.messages.findIndex((_, i) => i > cursor && fromClient(i, "shutdown"));
        if (at < 0) return { pass: false, detail: "the client never sent shutdown" };
        const response = responseTo(line, at);
        const actual = {
          answered: response != null && response.error == null && response.result == null,
          exitNotification: line.messages.some((_, i) => i > at && fromClient(i, "exit")),
          exitCode: exit?.code ?? null,
        };
        const expected = { answered: true, exitNotification: true, exitCode: step.expect.exitCode };
        return isDeepStrictEqual(actual, expected)
          ? { pass: true, detail: "", at }
          : { pass: false, detail: mismatch("shutdown", expected, actual) };
      }
    }
  };

  for (const step of scenario.steps) {
    const verdict = judge(step);
    if (verdict.at != null) {
      cursor = verdict.at;
      floor = verdict.state ?? -1;
    }
    results.push({ id: step.id, label: step.label, pass: verdict.pass, detail: verdict.detail });
  }
  return { session, messages: line.messages.length, steps: results };
}
