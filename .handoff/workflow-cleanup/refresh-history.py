#!/usr/bin/env python3
"""refresh-history.py owner... : re-pin workflow digest and executable closure in
tools/support/ci/workflow-program-history.json (run from repo root). Programs that no
longer exist are dropped; new driver files of the workflow are added."""
import hashlib, json, os, sys, glob
p='tools/support/ci/workflow-program-history.json'
d=json.load(open(p))
dig=lambda b: hashlib.sha256(b).hexdigest()
for owner in sys.argv[1:]:
    t=d[owner]
    t['workflow']=dig(open(f'.github/workflows/{owner}.yml','rb').read())
    progs=[f for f in t['programs'] if os.path.exists(f)]
    own=f'.github/scripts/workflows/{owner}/'
    if os.path.isdir(own):
        progs=sorted(set(progs)|{f for f in glob.glob(own+'*') if os.path.isfile(f)})
    t['programs']=progs
    h=hashlib.sha256()
    for f in progs: h.update((f+'\0'+dig(open(f,'rb').read())+'\n').encode())
    t['closure']=h.hexdigest()
    print(owner,len(progs))
open(p,'w').write(json.dumps(d,indent=2)+'\n')
