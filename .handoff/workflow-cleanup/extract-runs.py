#!/usr/bin/env python3
"""extract-runs.py <violations.txt> [stem ...]

Move long inline `run` commands of workflow steps into named Pkl shell programs.
Each violation line is `.github/workflows/<stem>.yml / <job> / <step name>: ...`.
The step's `run = "..."` (or `ci.command("<name>", "...")`) in the workflow's Pkl
sources becomes `ci.workflowScript("<slug>.sh")`, and the command is written to
`tools/config/pkl/scripts/.github/scripts/workflows/<stem>/<slug>.sh.pkl`, wrapped one
argument group per line. Steps whose job has no VIZE_CI_FILES get a step-level env.
Prints the sites it could not convert.
"""
import glob, os, re, sys
import yaml  # PyYAML is in the dev shell; fall back to a tiny reader if missing.

violations = [l.strip() for l in open(sys.argv[1]) if l.strip()]
only = set(sys.argv[2:])
sites = {}
for line in violations:
    m = re.match(r"\.github/workflows/([a-z0-9-]+)\.yml / ([^/]+?) / (.+?): (command longer|more than two)", line)
    if not m:
        continue
    stem, job, step = m.groups()[:3]
    if only and stem not in only:
        continue
    sites.setdefault((stem, job, step), True)


def slug(name):
    words = re.sub(r"[^a-z0-9]+", " ", name.lower()).split()
    out = []
    for w in words:
        if len("-".join(out + [w])) > 56:
            break
        out.append(w)
    return "-".join(out) or "step"


def unescape(text):
    return re.sub(r'\\(["\\tn])', lambda m: {"t": "\t", "n": "\n"}.get(m.group(1), m.group(1)), text)


def wrap(command):
    words = re.findall(r"""(?:[^\s'"]+|'[^']*'|"(?:[^"\\]|\\.)*")+""", command)
    if len(command) <= 100 or " ".join(words) != command:
        return command
    lines, current = [], words[0]
    test_list = words[:2] == ["node", "--test"]
    for word in words[1:]:
        breaks = (test_list and re.search(r"\.test\.[cm]?[jt]s$", word)) or (word.startswith("--") and not test_list) or word in ("--", "&&", "||")
        if breaks and len(current) > 0 and not current.endswith(" --"):
            lines.append(current)
            current = "  " + word
        else:
            current += " " + word
    lines.append(current)
    return " \\\n".join(lines)


def program(command):
    body = "\n".join(wrap(line) for line in command.split("\n"))
    indented = "\n".join("  " + l for l in body.split("\n"))
    return ('amends "../../../../../ShellScript.pkl"\n\ninterpreter = "bash"\n\ncommands {\n  #"""\n'
            + indented + '\n  """#\n}\n')


failed = []
for (stem, job, step) in sites:
    workflow = yaml.safe_load(open(f".github/workflows/{stem}.yml"))
    jobdef = workflow["jobs"].get(job, {})
    has_files = "VIZE_CI_FILES" in (workflow.get("env") or {}) or "VIZE_CI_FILES" in (jobdef.get("env") or {})
    # Staged scripts export VIZE_CI_FILES through GITHUB_ENV.
    has_files = has_files or any(str(st.get("uses", "")).endswith("stage-workflow-scripts") for st in jobdef.get("steps") or [])
    sources = glob.glob(f"tools/config/pkl/workflows/*/{stem}.pkl") + glob.glob(f"tools/config/pkl/parts/{stem}/*.pkl")
    name = slug(step)
    target_dir = f"tools/config/pkl/scripts/.github/scripts/workflows/{stem}"
    if os.path.exists(f"{target_dir}/{name}.sh.pkl"):
        name += "-2"
    done = False
    q = re.escape(step)
    env_line = (f'        env {{ VIZE_CI_FILES = "${{{{ github.workspace }}}}/.github/scripts/workflows/{stem}" }}\n')
    for source in sources:
        text = open(source).read()
        # new Dynamic { name = "<step>" ... run = "..." }
        block = re.compile(rf'(?P<indent> *)name = "{q}"\n(?P<mid>(?:(?!\1name = )\1[^\n]*\n)*?)(?P<rindent>\1)run = (?:"(?P<cmd>(?:[^"\\]|\\.)*)"|#"(?P<raw>.*?)"#|"""\n(?P<multi>(?:(?! *""").*\n)*?) *""")\n')
        m = block.search(text)
        raw = m and (m.group("raw") if m.group("raw") is not None else m.group("multi"))
        ok = m and ((m.group("cmd") is not None and "\\(" not in m.group("cmd")) or (raw is not None and "\\#(" not in raw))
        if ok:
            if m.group("cmd") is not None:
                command = unescape(m.group("cmd"))
            else:
                command = "\n".join(l.strip() for l in raw.strip("\n").split("\n") if l.strip())
            indent = m.group("rindent")
            mid = m.group("mid")
            extra = ""
            if not has_files and "VIZE_CI_FILES" not in mid:
                files_value = f'VIZE_CI_FILES = "${{{{ github.workspace }}}}/.github/scripts/workflows/{stem}"'
                one = re.search(rf"^{indent}env \{{ (.*) \}}$", mid, re.M)
                if one:
                    mid = mid[:one.start()] + f"{indent}env {{\n{indent}  {one.group(1)}\n{indent}  {files_value}\n{indent}}}" + mid[one.end():]
                elif re.search(rf"^{indent}env \{{$", mid, re.M):
                    mid = re.sub(rf"^({indent}env \{{)$", lambda e: e.group(1) + f"\n{indent}  {files_value}", mid, count=1, flags=re.M)
                else:
                    extra = f"{indent}env {{ {files_value} }}\n"
            replacement = (f'{m.group("indent")}name = "{step}"\n{mid}{extra}'
                           f'{indent}run = ci.workflowScript("{name}.sh")\n')
            text = text[:m.start()] + replacement + text[m.end():]
            done = True
        else:
            call = re.compile(rf'(?P<indent> *)ci\.command\("{q}", (?:"(?P<cmd>(?:[^"\\]|\\.)*)"|#"(?P<raw>.*?)"#)\)\n')
            m = call.search(text)
            if m and (m.group("cmd") is not None and "\\(" not in m.group("cmd") or m.group("raw") is not None and "\\#(" not in m.group("raw")):
                command = unescape(m.group("cmd")) if m.group("cmd") is not None else m.group("raw")
                i = m.group("indent")
                if has_files:
                    replacement = f'{i}ci.command("{step}", ci.workflowScript("{name}.sh"))\n'
                else:
                    replacement = (f'{i}new Dynamic {{\n{i}  name = "{step}"\n'
                                   + env_line.replace("        ", i + "  ", 1)
                                   + f'{i}  run = ci.workflowScript("{name}.sh")\n{i}}}\n')
                text = text[:m.start()] + replacement + text[m.end():]
                done = True
        if done:
            open(source, "w").write(text)
            os.makedirs(target_dir, exist_ok=True)
            open(f"{target_dir}/{name}.sh.pkl", "w").write(program(command))
            print(f"converted {stem} / {job} / {step} -> {name}.sh", file=sys.stderr)
            break
    if not done:
        failed.append(f"{stem} / {job} / {step}")
print("\n".join(failed))
