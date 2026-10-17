" Real vim-lsp transport, canonical Vite settings and per-source TS ownership.
set nocompatible
set nomore
set hidden
execute 'set runtimepath^=' . fnameescape($VIZE_TEST_VIM_LSP_PATH)
execute 'set runtimepath^=' . fnameescape($VIZE_E2E_PLUGIN_ROOT)
runtime plugin/lsp.vim
runtime plugin/vize.vim
runtime ftdetect/vize.vim
let s:root = $VIZE_E2E_WORKSPACE
let s:helper = s:root . '/vue-policy.mjs'
let s:source = ['<template>', '<div :title.sync="title" />', '</template>']
let s:valid_policy = 'export default { languageServer: { lint:true, typecheck:false, ecosystem:false, hover:false, formatting:false }, linter:{ preset:"incremental", rules:{ "vue/no-deprecated-v-bind-sync":"error" } } };'
call writefile(['{}'], s:root . '/package.json')
call writefile(['import vue from "./vue-policy.mjs"; export default { vue };'], s:root . '/vite.config.mjs')
call writefile([s:valid_policy], s:helper)
call writefile(['{"files":[],"references":[{"path":"./tsconfig.legacy.json"},{"path":"./tsconfig.modern.json"}]}'], s:root . '/tsconfig.json')
for [s:name, s:target] in [['legacy', 2.7], ['modern', 3.5]]
  call mkdir(s:root . '/src/' . s:name, 'p')
  call writefile([json_encode({'include':['src/' . s:name . '/**/*.vue'],'vueCompilerOptions':{'target':s:target}})], s:root . '/tsconfig.' . s:name . '.json')
  call writefile(s:source, s:root . '/src/' . s:name . '/App.vue')
endfor
call mkdir(s:root . '/packages/stable', 'p')
call writefile(['{}'], s:root . '/packages/stable/package.json')
call writefile(['export default { vue: { languageServer:{lint:false,typecheck:false,ecosystem:false,formatting:false} } };'], s:root . '/packages/stable/vite.config.mjs')
call writefile(s:source, s:root . '/packages/stable/Stable.vue')

" Saving before Vize starts must neither launch it nor send another server events.
execute 'edit ' . fnameescape(s:helper)
write
call assert_equal('unknown server', lsp#get_server_status('vize'))
call vize#setup({'cmd':[$VIZE_E2E_NODE,$VIZE_E2E_TAP,$VIZE_E2E_SERVER,'lsp']})
call lsp#enable()
execute 'edit ' . fnameescape(s:root . '/src/legacy/App.vue')
call assert_equal(0, lsp#utils#_wait(120000, {-> lsp#get_server_status('vize') ==# 'running'}, 50))
let s:legacy = lsp#utils#get_buffer_uri()

function! s:params(uri) abort
  let l:all = lsp#internal#diagnostics#state#_get_all_diagnostics_grouped_by_server_for_uri(a:uri)
  return get(get(l:all, 'vize', {}), 'params', {})
endfunction

function! s:expect_diagnostics(uri, version, diagnostics) abort
  let l:expected = {'uri':a:uri,'version':a:version,'diagnostics':a:diagnostics}
  call assert_equal(0, lsp#utils#_wait(30000, {-> s:params(a:uri) ==# l:expected}, 50), 'complete diagnostic publication')
  call assert_equal(l:expected, s:params(a:uri))
endfunction

function! s:packets() abort
  return map(readfile($VIZE_CONFORMANCE_TRANSCRIPT),'json_decode(v:val)')
endfunction

function! s:modern_publications() abort
  return len(filter(s:packets(),'get(v:val,"dir","") ==# "s2c" && get(get(v:val,"msg",{}),"method","") ==# "textDocument/publishDiagnostics" && get(v:val.msg.params,"uri","") ==# s:modern && get(v:val.msg.params,"version",-1) == 2'))
endfunction

let s:responses = {}
function! s:on_response(method, data) abort
  let s:responses[a:method] = a:data
endfunction

function! s:request(method, uri, expected) abort
  call lsp#send_request('vize', {'bufnr':bufnr(lsp#utils#uri_to_path(a:uri)),'method':a:method,'params':{'textDocument':{'uri':a:uri},'position':{'line':1,'character':8},'options':{'insertSpaces':v:true,'tabSize':2}},'on_notification':function('s:on_response',[a:method])})
  call assert_equal(0, lsp#utils#_wait(30000, {-> has_key(s:responses,a:method)}, 50), a:method . ' response')
  if has_key(s:responses,a:method)
    let l:packet = remove(s:responses,a:method).response
    call assert_false(has_key(l:packet,'error'), string(l:packet))
    call assert_true(has_key(l:packet,'result'))
    call assert_equal(a:expected,get(l:packet,'result','missing'), a:uri . ' ' . a:method)
  endif
endfunction

let s:modern_diagnostics = [{
      \ 'code':'vue/no-deprecated-v-bind-sync',
      \ 'codeDescription':{'href':'https://eslint.vuejs.org/rules/no-deprecated-v-bind-sync.html'},
      \ 'message':"the .sync modifier on v-bind was removed in Vue 3; use v-model:<prop> instead\n\nHelp: Vue 3 removed the .sync modifier and folded its two-way binding into argumented v-model. Replace it with v-model:<prop> (e.g. :title.sync=\"title\" -> v-model:title=\"title\").",
      \ 'range':{'start':{'line':1,'character':5},'end':{'line':1,'character':24}},
      \ 'severity':1,'source':'vize/lint',
      \ }]
call s:expect_diagnostics(s:legacy,1,[])
call s:request('textDocument/formatting',s:legacy,v:null)
call s:request('textDocument/hover',s:legacy,v:null)
execute 'edit ' . fnameescape(s:root . '/src/modern/App.vue')
let s:modern = lsp#utils#get_buffer_uri()
call s:expect_diagnostics(s:modern,1,s:modern_diagnostics)
call setline(2,'<div :title.sync="draft" />')
call s:request('textDocument/formatting',s:modern,v:null)
call s:expect_diagnostics(s:modern,2,s:modern_diagnostics)
execute 'edit ' . fnameescape(s:root . '/packages/stable/Stable.vue')
let s:stable = lsp#utils#get_buffer_uri()
call s:expect_diagnostics(s:stable,1,[])

" External edits are not claimed: only the plugin's actual BufWritePost bridge.
let s:outside = tempname() . '.mjs'
execute 'edit ' . fnameescape(s:outside)
call setline(1,'export default {};')
write
call delete(s:outside)
execute 'edit ' . fnameescape(s:helper)
let s:before_reload = s:modern_publications()
call setline(1,'export default {')
write
call assert_equal(0,lsp#utils#_wait(30000,{-> s:modern_publications() > s:before_reload},50),'invalid helper reload publishes the retained dirty document')
call s:request('textDocument/formatting',s:modern,v:null)
call s:expect_diagnostics(s:modern,2,s:modern_diagnostics)
call s:request('textDocument/formatting',s:stable,v:null)
let s:before_reload = s:modern_publications()
call setline(1,substitute(s:valid_policy,'formatting:false','formatting:true',''))
write
call assert_equal(0,lsp#utils#_wait(30000,{-> s:modern_publications() > s:before_reload},50),'valid helper reload recovers the dirty document')
let s:formatted = [{'newText':"<template>\n  <div :title.sync=\"draft\" />\n</template>\n",'range':{'start':{'line':0,'character':0},'end':{'line':3,'character':0}}}]
call s:request('textDocument/formatting',s:modern,s:formatted)
call s:expect_diagnostics(s:legacy,1,[])
call s:expect_diagnostics(s:modern,2,s:modern_diagnostics)
call s:request('textDocument/formatting',s:stable,v:null)
call assert_equal('<div :title.sync="title" />',readfile(s:root . '/src/modern/App.vue')[1], 'disk stays unchanged')
call assert_equal('<div :title.sync="draft" />',getbufline(bufnr(s:root . '/src/modern/App.vue'),2)[0], 'unsaved buffer survives reload')

call lsp#stop_server('vize')
call assert_equal(0,lsp#utils#_wait(10000,{-> lsp#get_server_status('vize') ==# 'exited'},50))
write
let s:packets = s:packets()
let s:initializations = filter(copy(s:packets),'get(v:val,"dir","") ==# "c2s" && get(get(v:val,"msg",{}),"method","") ==# "initialize"')
call assert_equal(1,len(s:initializations),'one real native server session')
call assert_equal({},get(s:initializations[0].msg.params.capabilities.workspace,'didChangeWatchedFiles',{}),'pinned client has no dynamic watch capability')
let s:changes = map(filter(copy(s:packets),'get(v:val,"dir","") ==# "c2s" && get(get(v:val,"msg",{}),"method","") ==# "workspace/didChangeWatchedFiles"'),'v:val.msg')
let s:event = {'jsonrpc':'2.0','method':'workspace/didChangeWatchedFiles','params':{'changes':[{'uri':lsp#utils#path_to_uri(s:helper),'type':2}]}}
call assert_equal([s:event,s:event],s:changes,'only two actual helper saves notify Vize')
if !empty(v:errors)
  call writefile(v:errors,$VIZE_E2E_ERROR_PATH)
  cquit 1
endif
quitall!
