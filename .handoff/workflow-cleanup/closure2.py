#!/usr/bin/env python3
"""closure2.py <candidates-file> : fast variant. One git grep per candidate, fixpoint in memory.
Prints DELETE <file> for files whose referrers are all deleted/historical/index files."""
import subprocess, sys, os, re
from concurrent.futures import ThreadPoolExecutor
cands=sorted({l.strip() for l in open(sys.argv[1]) if l.strip() and os.path.exists(l.strip())})
HIST=re.compile(r'^(docs/davinci/decisions/|docs/davinci/plan/|docs/release/2026-|tests/_fixtures/|.*\.yml\.gz$|.*\.before\.yml\.txt$|\.github/workflows/README\.md$)')
INDEX={'tests/tooling/fixtures/tooling-test-timings.json','tools/config/retired-tooling-languages.json'}
def referrers(f):
    name=os.path.basename(f); stem,ext=os.path.splitext(name)
    terms=[name]
    if ext in('.ts','.mjs','.js') and len(stem)>8: terms+=['/'+stem+'"',"/"+stem+"'"]
    args=[]
    for t in terms: args+=['-e',t]
    out=subprocess.run(['git','grep','-lF',*args],capture_output=True,text=True).stdout.split()
    return {o for o in out if o!=f and not HIST.match(o) and o not in INDEX}
with ThreadPoolExecutor(8) as ex: refs=dict(zip(cands,ex.map(referrers,cands)))
alive=set(cands); changed=True
while changed:
    changed=False
    for f in sorted(alive):
        if refs[f]-alive: alive.discard(f); changed=True
for f in sorted(alive): print('DELETE',f)
for f in sorted(set(cands)-alive): print('KEEP',f,'<-',' '.join(sorted(refs[f]-alive)[:6]))
