import { ensureTool } from "./core.mjs";
import { processFixRequest, processOpenFixRequests, watch } from "./workflow.mjs";

function usage() {
  console.log(`Usage:
  node tools/ai-fix-agent.mjs run --fix <number> [options]
  node tools/ai-fix-agent.mjs once [options]
  node tools/ai-fix-agent.mjs watch [options]

Options:
  --repo <owner/name>          GitHub repository. Defaults to gh repo view.
  --base <branch>             PR base branch. Defaults to repository default branch.
  --remote <name>             Git remote to push branches to. Default: origin.
  --fix <number>              Fix request number for run.
  --interval <seconds>        Watch polling interval. Default: 300.
  --limit <number>            Open fix request scan limit. Default: 50.
  --include-existing          Watch mode also processes fix requests opened before startup.
  --agent-command <command>   Shell command to run instead of the default Codex CLI command.
  --no-wait-ci                Create/update PR without waiting for checks.
  --help                      Show this help.

The default agent command is:
  codex exec --full-auto --cd <repo-root> -o <result-file> -

Custom commands receive these environment variables:
  AI_FIX_AGENT_CONTEXT_FILE
  AI_FIX_AGENT_PROMPT_FILE
  AI_FIX_AGENT_RESULT_FILE
  AI_FIX_AGENT_FIX_NUMBER
`);
}

function parseArgs(argv) {
  const args = { _: [] };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg.startsWith("--")) {
      args._.push(arg);
      continue;
    }

    if (arg === "--include-existing") {
      args.includeExisting = true;
      continue;
    }
    if (arg === "--no-wait-ci") {
      args.waitCi = false;
      continue;
    }
    if (arg === "--help") {
      args.help = true;
      continue;
    }

    const key = arg.slice(2).replaceAll("-", "_");
    const value = argv[i + 1];
    if (value == null || value.startsWith("--")) {
      throw new Error(`${arg} requires a value`);
    }
    args[key] = value;
    i += 1;
  }

  return args;
}

export async function main(argv) {
  const options = parseArgs(argv);
  const command = options._[0];

  if (options.help || command == null) {
    usage();
    return;
  }

  ensureTool("git");
  ensureTool("gh");
  if ((options.agent_command ?? process.env.AI_FIX_AGENT_COMMAND) == null) {
    ensureTool("codex");
  }

  if (command === "run") {
    if (options.fix == null) {
      throw new Error("run requires --fix <number>");
    }
    processFixRequest(options.fix, options);
  } else if (command === "once") {
    processOpenFixRequests({ ...options, includeExisting: true }, new Date(0));
  } else if (command === "watch") {
    await watch(options);
  } else {
    throw new Error(`Unknown command: ${command}`);
  }
}
