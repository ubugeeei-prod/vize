//! Test-only complete packet custody; provider discovery resolves paths without loading packages.
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

pub(super) fn digest(bytes: &[u8]) -> vize_l0::String {
    use std::fmt::Write;
    let mut value = vize_l0::String::default();
    for byte in Sha256::digest(bytes) {
        write!(&mut value, "{byte:02x}").unwrap();
    }
    value
}

pub(super) fn save(path: &Path, value: &Value) {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

pub(super) fn retain(directory: &Path, output: &Output, source: &str) {
    fs::create_dir_all(directory).unwrap();
    fs::write(directory.join("stdout.bin"), &output.stdout).unwrap();
    fs::write(directory.join("stderr.bin"), &output.stderr).unwrap();
    fs::write(directory.join("input-source.txt"), source).unwrap();
    let value = json!({"status":output.status.code(),"success":output.status.success(),
        "stdoutBytes":output.stdout.len(),"stderrBytes":output.stderr.len(),"source":source,
        "stdoutSha256":digest(&output.stdout),"stderrSha256":digest(&output.stderr)});
    save(&directory.join("process.json"), &value);
    println!("imported-raw-callback-process {}", value);
}

pub(super) fn providers(workspace: &Path, evidence: &Path, runtime: &Path) -> Value {
    let script = r#"
import fs from 'node:fs';
import path from 'node:path';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
const hash=b=>createHash('sha256').update(b).digest('hex');
const file=p=>{const b=fs.readFileSync(p);return {path:fs.realpathSync(p),bytes:b.length,sha256:hash(b),body:b.toString('utf8')}};
const req=createRequire(path.join(process.argv[1],'tests/package.json'));
function inspect(name, from=req) {
  const entry=fs.realpathSync(from.resolve(name));let root=path.dirname(entry);
  while (true) {
    const p=path.join(root,'package.json');
    if (fs.existsSync(p)) {
      const m=JSON.parse(fs.readFileSync(p,'utf8'));
      if (m.name===name) return {name,root,entry,entrySha256:file(entry).sha256,version:m.version,manifest:file(p),declaration:m.types?file(path.resolve(root,m.types)):null};
    }
    const parent=path.dirname(root);if(parent===root)throw Error('no package owner: '+name);root=parent;
  }
}
const vue=inspect('vue');
const dom=inspect('@vue/runtime-dom',createRequire(vue.entry));
const core=inspect('@vue/runtime-core',createRequire(dom.entry));
const reactivity=inspect('@vue/reactivity',createRequire(dom.entry));
const typescript=inspect('typescript');const vueTsc=inspect('vue-tsc');
const languageCore=inspect('@vue/language-core',createRequire(vueTsc.entry));
console.log(JSON.stringify({vue,dom,core,reactivity,typescript,vueTsc,languageCore,node:{version:process.version,executable:fs.realpathSync(process.execPath),sha256:hash(fs.readFileSync(process.execPath))},scope:'actual workspace distribution bodies; native loaded-file membership and historical Misskey declarations UNOBSERVED'}));
"#;
    let output = Command::new("node")
        .args(["--input-type=module", "-e", script])
        .arg(workspace)
        .output()
        .unwrap();
    retain(&evidence.join("provider-discovery"), &output, "");
    assert!(output.status.success(), "provider discovery: {:?}", output);
    let mut value: Value = serde_json::from_slice(&output.stdout).unwrap();
    let bytes = fs::read(runtime).unwrap();
    value["native"] = json!({"path":fs::canonicalize(runtime).unwrap(),"bytes":bytes.len(),"sha256":digest(&bytes),"sourceOwner":"UNJOINED"});
    let manifest_path = runtime
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("package.json");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    value["native"]["manifest"] = json!({"path":manifest_path,"body":manifest,"sha256":digest(manifest.as_bytes()),"package":serde_json::from_str::<Value>(&manifest).unwrap()});
    let cli = Path::new(env!("CARGO_BIN_EXE_vize"));
    let cli_bytes = fs::read(cli).unwrap();
    value["sourceCLI"] = json!({"path":fs::canonicalize(cli).unwrap(),"bytes":cli_bytes.len(),"sha256":digest(&cli_bytes),"crateVersion":env!("CARGO_PKG_VERSION")});
    save(&evidence.join("providers.json"), &value);
    println!(
        "imported-raw-callback-providers {}",
        json!({"file":evidence.join("providers.json"),"sha256":digest(&fs::read(evidence.join("providers.json")).unwrap())})
    );
    value
}

pub(super) fn forwarder(evidence: &Path) -> PathBuf {
    let directory = evidence.join("observer/.bin");
    fs::create_dir_all(&directory).unwrap();
    let source = r#"#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const image=fs.realpathSync(process.env.VIZE_RAW_NATIVE);
const directory=fs.mkdtempSync(path.join(process.env.VIZE_RAW_CAPTURE,'call-'));
const argv=process.argv.slice(2),cwd=fs.realpathSync(process.cwd());
const hash=b=>createHash('sha256').update(b).digest('hex');
const write=(p,v)=>fs.writeFileSync(path.join(directory,p),JSON.stringify(v,null,2)+'\n',{flag:'wx'});
const i=argv.indexOf('--project'),config=i>=0?path.resolve(cwd,argv[i+1]):null;
const kind=argv.includes('--checkers')?'main-cli':config?'option-probe':argv.length===1&&argv[0]==='--version'?'version':'UNKNOWN';
write('request.json',{argv,cwd,config,kind,executable:image,imageBefore:hash(fs.readFileSync(image)),node:{version:process.version,executable:fs.realpathSync(process.execPath)},scope:'test-only passive CLI forwarding; input closure and native loaded-file membership UNOBSERVED'});
if(config){try{fs.copyFileSync(config,path.join(directory,'config.bytes'))}catch(e){write('config-read-error.json',{error:String(e)})}}
const env={...process.env};delete env.VIZE_RAW_NATIVE;delete env.VIZE_RAW_CAPTURE;
const r=spawnSync(image,argv,{cwd,env,timeout:120000,maxBuffer:64*1024*1024});
fs.writeFileSync(path.join(directory,'native-replay.stdout.bin'),r.stdout??Buffer.alloc(0),{flag:'wx'});
fs.writeFileSync(path.join(directory,'native-replay.stderr.bin'),r.stderr??Buffer.alloc(0),{flag:'wx'});
write('native-replay.status.json',{status:r.status,signal:r.signal,error:r.error?.message??null,pid:r.pid});
try{write('image-after.json',{sha256:hash(fs.readFileSync(image))})}catch(e){write('image-read-error.json',{error:String(e)})}
if(r.stdout?.length)process.stdout.write(r.stdout);
if(r.stderr?.length)process.stderr.write(r.stderr);
if(r.signal)process.kill(process.pid,r.signal);else process.exitCode=r.status??1;
"#;
    let executable = directory.join("native-forward.mjs");
    fs::write(&executable, source).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
    }
    executable
}

pub(super) fn native_calls(directory: &Path) -> Value {
    let mut paths = fs::read_dir(directory)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect::<Vec<_>>();
    paths.sort();
    let rows = paths.into_iter().map(|path| {
        let read=|name|serde_json::from_slice::<Value>(&fs::read(path.join(name)).unwrap()).unwrap();
        let request=read("request.json");
        let mut included=Vec::new();
        if request["kind"]=="main-cli" {
            let config_path=Path::new(request["config"].as_str().unwrap());
            let config=read("config.bytes");
            for entry in config["include"].as_array().unwrap() {
                let physical=fs::canonicalize(config_path.parent().unwrap().join(entry.as_str().unwrap())).unwrap();
                let body=fs::read_to_string(&physical).unwrap();
                included.push(json!({"literal":entry,"physical":physical,"sha256":digest(body.as_bytes()),"body":body}));
            }
        }
        json!({"directory":path,"request":request,"status":read("native-replay.status.json"),
            "stdout":fs::read(path.join("native-replay.stdout.bin")).unwrap(),
            "stderr":fs::read(path.join("native-replay.stderr.bin")).unwrap(),"includedSourceBodies":included})
    }).collect::<Vec<_>>();
    let value = json!(rows);
    save(&directory.with_extension("json"), &value);
    println!(
        "imported-raw-callback-native-custody {}",
        json!({"file":directory.with_extension("json"),"sha256":digest(&fs::read(directory.with_extension("json")).unwrap())})
    );
    value
}

pub(super) fn geometry(evidence: &Path, providers: &Value, inputs: &Value) -> Value {
    let input = evidence.join("geometry-input.json");
    save(&input, inputs);
    let script = r#"
import fs from 'node:fs';import {createRequire} from 'node:module';
const req=createRequire(process.argv[1]);const ts=req(process.argv[1]);
const inputs=JSON.parse(fs.readFileSync(process.argv[2],'utf8'));
const rows=inputs.map(input=>{
 const sf=ts.createSourceFile('Parent.vue.ts',input.virtualTs,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);
 const imports=[],negative=[];
 const point=node=>{const p=sf.getLineAndCharacterOfPosition(node.getStart(sf));return {line:p.line+1,column:p.character+1}};
 const stringMarkers=new Set(['direct_bad','callback_bad']);const missingMarkers=new Set(['directMissing','callbackMissing']);
 const ancestor=(node,predicate)=>{for(let current=node;current;current=current.parent)if(predicate(current))return current;return null};
 const owner=node=>{
  const declaration=ancestor(node,ts.isVariableDeclaration);
  if(!declaration){const statement=ancestor(node,ts.isExpressionStatement);if(statement&&ts.isVoidExpression(statement.expression))return 'interpolation';throw new Error('unknown interpolation owner')}
  if(!ts.isIdentifier(declaration.name))throw new Error('non-identifier negative owner');
  if(declaration.name.text==='__vize_prop_check_0_textConverter')return 'checked-callback';
  if(declaration.name.text!=='__vize_resolved_Formatter_props_0')throw new Error('unknown callback owner');
  const property=ancestor(node,ts.isPropertyAssignment);
  if(!property||!ts.isStringLiteral(property.name)||property.name.text!=='textConverter')throw new Error('unknown inference property');
  const comments=(ts.getLeadingCommentRanges(input.virtualTs,property.pos)??[]).map(range=>input.virtualTs.slice(range.pos,range.end));
  if(JSON.stringify(comments)!==JSON.stringify(['// @ts-ignore Inference-only callback prop; mapped prop owner checks diagnostics.']))throw new Error('unguarded inference copy');
  if(point(property).line!==point(node).line)throw new Error('inference diagnostic escaped the single ignored line');
  return 'inference-only';
 };
 function visit(node){
  if(ts.isImportDeclaration(node)&&node.moduleSpecifier.text==='./Formatter.vue.ts')imports.push({...point(node.moduleSpecifier),text:node.moduleSpecifier.text});
  if(ts.isStringLiteral(node)&&stringMarkers.has(node.text)&&ts.isCallExpression(node.parent)&&ts.isPropertyAccessExpression(node.parent.expression)&&node.parent.expression.name.text==='repeat')negative.push({...point(node),owner:owner(node),marker:"'"+node.text+"'",code:2345,message:"Argument of type 'string' is not assignable to parameter of type 'number'."});
  if(ts.isPropertyAccessExpression(node)&&missingMarkers.has(node.name.text))negative.push({...point(node.name),owner:owner(node),marker:node.name.text,code:2339,message:"Property '"+node.name.text+"' does not exist on type 'Priority'."});
  ts.forEachChild(node,visit);
 }
 visit(sf);negative.sort((a,b)=>a.line-b.line||a.column-b.column);
 return {phase:input.phase,imports,negative,parseDiagnostics:sf.parseDiagnostics.map(d=>({code:d.code,start:d.start,length:d.length,messageText:d.messageText}))};
});console.log(JSON.stringify({typescript:ts.version,rows}));
"#;
    let output = Command::new("node")
        .args(["--input-type=module", "-e", script])
        .arg(providers["typescript"]["entry"].as_str().unwrap())
        .arg(&input)
        .output()
        .unwrap();
    retain(&evidence.join("geometry-process"), &output, "");
    assert!(output.status.success(), "{:?}", output);
    let value = serde_json::from_slice(&output.stdout).unwrap();
    save(&evidence.join("geometry.json"), &value);
    value
}
