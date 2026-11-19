import glob, os, sys, yaml
root = sys.argv[1]
rows = []
for path in sorted(glob.glob(f"{root}/.github/workflows/*.yml")):
    try:
        doc = yaml.safe_load(open(path))
    except Exception as e:
        continue
    on = doc.get(True) or doc.get("on") or {}
    if isinstance(on, str): on = {on: None}
    if isinstance(on, list): on = {k: None for k in on}
    if "pull_request" not in on: continue
    pr = on.get("pull_request") or {}
    paths = pr.get("paths") if isinstance(pr, dict) else None
    mq = "merge_group" in on
    push = on.get("push")
    pushmain = isinstance(push, dict) and "main" in (push.get("branches") or [])
    sched = "schedule" in on
    jobs = doc.get("jobs", {})
    runners = sorted({str(j.get("runs-on")) for j in jobs.values() if isinstance(j, dict)})
    rows.append((os.path.basename(path), len(paths) if paths else "ALL", mq, pushmain, sched, len(jobs), " ".join(runners)[:60]))
for r in rows: print(*r, sep="\t")
print(len(rows))
