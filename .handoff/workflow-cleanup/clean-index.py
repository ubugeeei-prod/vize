#!/usr/bin/env python3
"""Remove deleted files (staged deletions) from timing and retired-language indexes."""
import subprocess
gone=set(subprocess.run(['git','diff','--cached','--name-only','--diff-filter=D'],capture_output=True,text=True).stdout.split())
gone|=set(subprocess.run(['git','diff','--name-only','--diff-filter=D'],capture_output=True,text=True).stdout.split())
import json
for p in ['tests/tooling/fixtures/tooling-test-timings.json','tools/config/retired-tooling-languages.json']:
    lines=open(p).read().split('\n')
    out=[l for l in lines if not any(('"'+g+'"') in l for g in gone)]
    # fix trailing commas before closing brackets
    fixed=[]
    for i,l in enumerate(out):
        nxt=next((x for x in out[i+1:] if x.strip()),'')
        if l.rstrip().endswith(',') and nxt.strip()[:1] in (']','}'):
            l=l.rstrip()[:-1]
        fixed.append(l)
    open(p,'w').write('\n'.join(fixed)); json.loads(open(p).read())
    print(p,len(lines)-len(out))
