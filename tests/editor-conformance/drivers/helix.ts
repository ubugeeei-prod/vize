// Real Helix (pinned in CI) in a pseudo-terminal, driven by keystrokes: the
// packaged `editors/helix/languages.toml` is the user's languages.toml (plus
// the formatting opt-in every client makes for this scenario), `vize` resolves
// through PATH as the package expects, and every request is one Helix builds
// from its own cursor, selection, menu and prompt. The driver waits on the tap
// transcript between keystrokes; it never reads a response itself.
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

import {
  repoRoot,
  settleDiagnostics,
  step,
  suiteRoot,
  transcriptLength,
  type Driver,
  type DriverContext,
} from "../support/context.ts";
import { answeredRequest, readTranscript, waitForTranscript } from "../support/transcript.ts";

const hx = process.env.VIZE_TEST_HELIX_PATH ?? "hx";
const ESC = "\x1b";
const ENTER = "\r";

function version(): string {
  const output = spawnSync(hx, ["--version"], { encoding: "utf8" }).stdout ?? "";
  return /^helix (\S+)/mu.exec(output)?.[1] ?? "unknown";
}

/** Terminal output without its escape sequences (CSI, OSC, charset) and control bytes. */
function visibleText(raw: string): string {
  let text = "";
  for (let at = 0; at < raw.length; at += 1) {
    const code = raw.charCodeAt(at);
    if (code === 0x1b) {
      const kind = raw[at + 1];
      at += 1;
      if (kind === "[") {
        do at += 1;
        while (at < raw.length && (raw.charCodeAt(at) < 0x40 || raw.charCodeAt(at) > 0x7e));
      } else if (kind === "]") {
        do at += 1;
        while (at < raw.length && raw.charCodeAt(at) !== 0x07 && raw.charCodeAt(at) !== 0x1b);
      } else if (kind === "(" || kind === ")") {
        at += 1;
      }
      text += " ";
    } else if (code >= 0x20 || code === 0x0a) {
      text += raw[at];
    }
  }
  return text.replace(/ {2,}/gu, " ");
}

/** The packaged languages.toml with `formatting = true` added to the server config table. */
function userLanguages(): string {
  const packaged = fs.readFileSync(
    path.join(repoRoot, "editors", "helix", "languages.toml"),
    "utf8",
  );
  const table = "[language-server.vize.config]\n";
  if (!packaged.includes(table))
    throw new Error("editors/helix/languages.toml lost its config table");
  return packaged.replace(table, `${table}formatting = true\n`);
}

async function run(context: DriverContext): Promise<void> {
  const { scenario } = context;
  const config = path.join(context.scratch, "config");
  fs.mkdirSync(path.join(config, "helix"), { recursive: true });
  fs.writeFileSync(path.join(config, "helix", "languages.toml"), userLanguages());
  // Completion only when asked (C-x), no signature/inlay requests racing the scenario.
  fs.writeFileSync(
    path.join(config, "helix", "config.toml"),
    "[editor]\nauto-completion = false\n\n[editor.lsp]\nauto-signature-help = false\ndisplay-inlay-hints = false\n",
  );
  const screen = path.join(context.scratch, "helix-screen.log");
  const editor = spawn(
    "python3",
    [
      path.join(suiteRoot, "drivers", "pty-bridge.py"),
      "--cols",
      "120",
      "--rows",
      "40",
      "--screen",
      screen,
      "--",
      hx,
      scenario.document,
    ],
    {
      cwd: context.workspace,
      env: {
        ...context.env,
        TERM: "xterm-256color",
        XDG_CONFIG_HOME: config,
        XDG_CACHE_HOME: path.join(context.scratch, "cache"),
        XDG_DATA_HOME: path.join(context.scratch, "data"),
      },
      stdio: ["pipe", "inherit", "inherit"],
    },
  );
  const exited = new Promise<number | null>((resolve) => editor.on("close", resolve));
  // Every wait also ends (as a failure) if Helix exits underneath it.
  const early = exited.then((code) => {
    throw new Error(`Helix exited early with ${code}`);
  });
  early.catch(() => {});
  const guard = <T>(promise: Promise<T>) => Promise.race([promise, early]);
  const keys = async (sequence: string) => {
    if (editor.exitCode != null) throw new Error(`Helix exited early with ${editor.exitCode}`);
    editor.stdin.write(sequence);
    // Let Helix tell a lone Escape from an Alt chord before the next key.
    await sleep(sequence.endsWith(ESC) ? 300 : 100);
  };
  const goto = (position: { line: number; character: number }) =>
    keys(
      `:${position.line + 1}${ENTER}gh${position.character > 0 ? `${position.character}l` : ""}`,
    );
  const answered = (method: string, after: number) =>
    guard(
      waitForTranscript(context.transcript, `a ${method} answer`, (entries) =>
        answeredRequest(entries, method, after),
      ),
    );
  const settle = (after: number, id: string) =>
    guard(settleDiagnostics(context, after, step(scenario, id, "diagnostics").expect.length));

  try {
    await guard(
      waitForTranscript(context.transcript, "didOpen", (entries) =>
        entries.some((entry) => entry.msg?.method === "textDocument/didOpen") ? true : undefined,
      ),
    );
    await settle(0, "diagnostics-open");

    let mark = transcriptLength(context);
    await goto(step(scenario, "hover", "request").match.position);
    await keys(" k");
    await answered("textDocument/hover", mark);
    await keys(ESC);

    mark = transcriptLength(context);
    await goto(step(scenario, "completion", "request").match.position);
    await keys("i\x18");
    await answered("textDocument/completion", mark);
    await keys(ESC);
    await keys(ESC);

    mark = transcriptLength(context);
    await goto(step(scenario, "definition", "request").match.position);
    await keys("gd");
    await answered("textDocument/definition", mark);
    await sleep(500);
    await keys("\x0f"); // C-o: back to the scenario document

    mark = transcriptLength(context);
    const range = step(scenario, "code-action", "request").match.range;
    await goto(range.start);
    await keys(" a");
    await answered("textDocument/codeAction", mark);
    await sleep(500);
    mark = transcriptLength(context);
    await keys(ENTER); // Helix lists the preferred quick fix first
    await settle(mark, "diagnostics-quick-fix");

    mark = transcriptLength(context);
    await keys(`:format${ENTER}`);
    await answered("textDocument/formatting", mark);
    await sleep(500);

    const edit = step(scenario, "edit", "text").action as { replace: any; text: string };
    const width = edit.replace.end.character - edit.replace.start.character;
    mark = transcriptLength(context);
    await goto(edit.replace.start);
    await keys(`v${width - 1}lc${edit.text}${ESC}`);
    await settle(mark, "diagnostics-edit");

    const rename = step(scenario, "rename", "request").match;
    mark = transcriptLength(context);
    await goto(rename.position);
    await keys(" r");
    await answered("textDocument/prepareRename", mark);
    await sleep(500);
    await keys(`\x15${rename.newName}${ENTER}`); // C-u clears the prefilled name
    await answered("textDocument/rename", mark);
    await sleep(1000);

    await keys(`:qa!${ENTER}`);
    const code = await Promise.race([exited, sleep(30_000).then(() => "timeout" as const)]);
    if (code !== 0) throw new Error(`Helix exited with ${code}`);
  } catch (error) {
    editor.kill("SIGKILL");
    // What Helix last drew, with terminal control sequences stripped.
    const drawn = visibleText(fs.existsSync(screen) ? fs.readFileSync(screen, "utf8") : "").slice(
      -3000,
    );
    const tail = readTranscript(context.transcript)
      .slice(-5)
      .map((entry) => JSON.stringify(entry).slice(0, 300));
    throw new Error(
      `${String(error)}\n--- last transcript entries ---\n${tail.join("\n")}\n--- screen tail ---\n${drawn}`,
    );
  }
}

export const driver: Driver = { client: "Helix", version, mode: "editor", run };
