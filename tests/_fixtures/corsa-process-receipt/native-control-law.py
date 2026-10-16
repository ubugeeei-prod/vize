#!/usr/bin/env python3
"""Supplemental actual LSP transport law; full source-built CLI acceptance is separate."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import queue
import subprocess
import threading
import time

parser = argparse.ArgumentParser(description="Actual native transport custody law; no CLI parity credit")
parser.add_argument("--native", type=Path, required=True)
parser.add_argument("--output", type=Path, required=True)
args = parser.parse_args()
FIXTURE = Path(__file__).resolve().parents[1] / "differential/typechecker/native-semantic-completion"
HELPER = Path(__file__).with_name("native-control.py")
NATIVE = args.native.resolve(strict=True)
OUTPUT = args.output
OUTPUT.mkdir(exist_ok=False)
case = json.loads((FIXTURE / 'case.json').read_text())
version = subprocess.run([NATIVE, '--version'], capture_output=True, check=False)
(OUTPUT / 'identity.json').write_text(json.dumps({'native':str(NATIVE), 'sha256': hashlib.sha256(NATIVE.read_bytes()).hexdigest(), 'versionStdoutHex':version.stdout.hex(), 'versionStderrHex':version.stderr.hex(), 'versionStatus':version.returncode, 'helperSha256':hashlib.sha256(HELPER.read_bytes()).hexdigest()}, indent=2))
assert version.returncode == 0 and version.stdout == b'Version 7.0.2\n' and version.stderr == b''

for mode in ['unexpected', 'nonutf8', 'nonzero', 'sigsegv', 'timeout']:
    root = OUTPUT / mode / 'frozen'
    root.mkdir(parents=True)
    for item in case['inputFiles']:
        target = root / item['file']
        target.parent.mkdir(parents=True, exist_ok=True)
        data = (FIXTURE / item['fixture']).read_bytes()
        assert hashlib.sha256(data).hexdigest() == item['sha256'] and len(data) == item['bytes']
        target.write_bytes(data)
    custody = OUTPUT / mode / 'control'
    argv = ['python3',str(HELPER),'--native',str(NATIVE),'--mode',mode,'--control-dir',str(custody),'--','--lsp','--stdio']
    child = subprocess.Popen(argv,cwd=root,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE,bufsize=0)
    lock = threading.Lock()
    messages = queue.Queue()
    raw_out = bytearray()
    raw_err = bytearray()
    events=[]

    def send(message):
        body=json.dumps(message,separators=(',',':')).encode()
        frame=b'Content-Length: '+str(len(body)).encode()+b'\r\n\r\n'+body
        with lock:
            child.stdin.write(frame)
            events.append({'sent':message,'frameHex':frame.hex()})

    def reader():
        while True:
            header=bytearray()
            while not header.endswith(b'\r\n\r\n'):
                part=child.stdout.read(1)
                if not part: return
                header.extend(part)
            length=int(header.split(b':',1)[1].split(b'\r\n',1)[0])
            body=bytearray()
            while len(body)<length: body.extend(child.stdout.read(length-len(body)))
            raw_out.extend(header+body)
            message=json.loads(body)
            events.append({'received':message,'frameHex':bytes(header+body).hex()})
            if 'id' in message and 'method' in message:
                result=[None]*len(message['params']['items']) if message['method']=='workspace/configuration' else None
                send({'jsonrpc':'2.0','id':message['id'],'result':result})
            else: messages.put(message)

    def errors():
        while part:=child.stderr.read(65536): raw_err.extend(part)

    threads=[threading.Thread(target=reader),threading.Thread(target=errors)]
    for thread in threads: thread.start()

    def request(number,method,params=None,omitted=False):
        message={'jsonrpc':'2.0','id':number,'method':method}
        if not omitted: message['params']=params
        send(message)
        until=time.monotonic()+15
        while time.monotonic()<until:
            result=messages.get(timeout=max(.01,until-time.monotonic()))
            if result.get('id')==number:
                assert 'error' not in result,result
                return result['result']
        raise TimeoutError(method)

    uri=root.as_uri()
    init=request(1,'initialize',{'processId':os.getpid(),'rootUri':uri,'workspaceFolders':[{'uri':uri,'name':'frozen'}],'capabilities':{'workspace':{'didChangeWatchedFiles':{'dynamicRegistration':True,'relativePatternSupport':True}},'textDocument':{'diagnostic':{'dynamicRegistration':False,'relatedDocumentSupport':True}}}})
    send({'jsonrpc':'2.0','method':'initialized','params':{}})
    shared=root/'shared.ts'
    send({'jsonrpc':'2.0','method':'textDocument/didOpen','params':{'textDocument':{'uri':shared.as_uri(),'languageId':'typescript','version':1,'text':shared.read_text()}}})
    diagnostic=request(2,'textDocument/diagnostic',{'textDocument':{'uri':shared.as_uri()}})
    time.sleep(0.5)  # Complete actual watcher/configuration response work before shutdown.
    request(3,'shutdown',omitted=True)
    send({'jsonrpc':'2.0','method':'exit'})
    child.stdin.close()
    timed_out=False
    try:
        status=child.wait(timeout=2 if mode=='timeout' else 15)
    except subprocess.TimeoutExpired:
        timed_out=True
        child.kill()
        status=child.wait(timeout=3)
    for thread in threads: thread.join(timeout=5)
    assert all(not thread.is_alive() for thread in threads)
    (OUTPUT/mode/'protocol.json').write_text(json.dumps({'argv':argv,'cwd':str(root),'wrapperPid':child.pid,'status':status,'events':events,'diagnostic':diagnostic},indent=2))
    (OUTPUT/mode/'stdout.bin').write_bytes(raw_out)
    (OUTPUT/mode/'stderr.bin').write_bytes(raw_err)
    receipt=json.loads((custody/'control.json').read_text())
    expected=(b'unexpected native stderr\n' if mode=='unexpected' else b'wrapper\0\xffcontrol\n' if mode=='nonutf8' else b'')+b"error handling method 'exit': EOF\n"
    expected_status={'nonzero':17,'sigsegv':-11,'timeout':-9}.get(mode,0)
    assert status==expected_status,(mode,status,receipt)
    assert timed_out==(mode=='timeout')
    assert raw_err==expected,(mode,bytes(raw_err))
    assert receipt['nativeExit']==0 and receipt['nativeReaped'] and receipt['shutdownAcknowledged']
    assert receipt['errors']==[] and not receipt['timedOut'] and not receipt['killRequested']
    assert all(receipt[key] for key in ['stdinComplete','stdoutComplete','stderrComplete','exitTranslatedToNativeEof'])
    assert receipt['nativePid']!=receipt['wrapperPid']==child.pid
    assert (custody/'native-child.txt').read_text() == (
        f"wrapper-pid={child.pid}\nnative-pid={receipt['nativePid']}\n"
        f"native-exit=0\nwrapper-exit={None if mode in ('sigsegv','timeout') else 17 if mode=='nonzero' else 0}\n"
    )
    assert receipt['wrapperSourceSha256'] == hashlib.sha256(HELPER.read_bytes()).hexdigest()
    assert (custody/'native.stderr.bin').read_bytes()==b''
    assert (custody/'native.stdout.bin').read_bytes()==(custody/'wrapper.stdout.bin').read_bytes()==raw_out
    assert (custody/'native.stdin.bin').read_bytes()+(custody/'translated-exit.frames.bin').read_bytes()==(custody/'wrapper.stdin.bin').read_bytes()
    assert diagnostic == {
        "kind": "full", "items": [{
            "range": {"start": {"line": 0, "character": 20}, "end": {"line": 0, "character": 30}},
            "severity": 1, "code": 2304, "source": "ts", "message": "Cannot find name 'LeafGlobal'.",
        }],
    }, diagnostic
    for item in case['inputFiles']: assert (root/item['file']).read_bytes()==(FIXTURE/item['fixture']).read_bytes()
    print(mode,status,'native',receipt['nativeExit'],'full config TS2304 retained')
print(OUTPUT)
