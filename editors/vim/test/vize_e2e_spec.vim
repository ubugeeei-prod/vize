" Headless Vim end-to-end scenario against a real `vize lsp` process through
" the packaged vim-lsp registration helper.

set nocompatible
set nomore

let s:plugin_root = $VIZE_E2E_PLUGIN_ROOT
let s:vim_lsp_root = $VIZE_TEST_VIM_LSP_PATH
let s:workspace = $VIZE_E2E_WORKSPACE
let s:server = $VIZE_E2E_SERVER

call assert_notequal('', s:plugin_root, 'VIZE_E2E_PLUGIN_ROOT must be set')
call assert_notequal('', s:vim_lsp_root, 'VIZE_TEST_VIM_LSP_PATH must be set')
call assert_notequal('', s:workspace, 'VIZE_E2E_WORKSPACE must be set')
call assert_notequal('', s:server, 'VIZE_E2E_SERVER must be set')

execute 'set runtimepath^=' . fnameescape(s:vim_lsp_root)
execute 'set runtimepath^=' . fnameescape(s:plugin_root)
runtime plugin/lsp.vim
runtime plugin/vize.vim
runtime ftdetect/vize.vim
execute 'source ' . fnameescape(s:plugin_root . '/test/vize_e2e_expected.vim')
let s:expected = g:vize_e2e_expected

let s:resolved = vize#setup({
      \ 'cmd': [s:server, 'lsp'],
      \ })
call assert_equal(['vue', 'art-vue'], s:resolved.allowlist, 'packaged filetypes')
call assert_equal({}, s:resolved.initialization_options, 'packaged project-owned feature settings')

" Register first, then let vim-lsp enable itself. This is the normal vimrc
" startup order and catches integrations that wrongly require an autoload
" function to have run before vize#setup().
call lsp#enable()
execute 'edit ' . fnameescape(s:workspace . '/src/Scenario.vue')
call assert_equal('vue', &filetype, 'ftdetect maps the fixture to vue')
call assert_equal(s:expected.authored_source, getline(1, '$'), 'authored fixture source')

let s:ready = lsp#utils#_wait(
      \ 120000,
      \ {-> lsp#get_server_status('vize') ==# 'running'},
      \ 100,
      \ )
call assert_equal(0, s:ready, 'the vize language server did not initialize')
let s:uri = lsp#utils#get_buffer_uri()
call assert_equal(
      \ resolve(s:workspace),
      \ resolve(lsp#utils#uri_to_path(lsp#get_server_root_uri('vize'))),
      \ 'workspace root',
      \ )

function! s:diagnostic_compare(left, right) abort
  let l:left = a:left.range.start
  let l:right = a:right.range.start
  if l:left.line != l:right.line
    return l:left.line - l:right.line
  endif
  return l:left.character - l:right.character
endfunction

function! s:diagnostics(uri) abort
  let l:grouped = lsp#internal#diagnostics#state#_get_all_diagnostics_grouped_by_server_for_uri(a:uri)
  let l:diagnostics = deepcopy(get(get(get(l:grouped, 'vize', {}), 'params', {}), 'diagnostics', []))
  return sort(l:diagnostics, function('s:diagnostic_compare'))
endfunction

let s:diagnostics_ready = lsp#utils#_wait(
      \ 240000,
      \ {-> s:diagnostics(s:uri) ==# s:expected.diagnostics},
      \ 200,
      \ )
call assert_equal(0, s:diagnostics_ready, 'real server did not publish the scenario diagnostics')
call assert_equal(s:expected.diagnostics, s:diagnostics(s:uri))

let s:responses = {}
function! s:on_response(method, data) abort
  let s:responses[a:method] = a:data
endfunction

function! s:request(method, params, expected) abort
  call lsp#send_request('vize', {
        \ 'method': a:method,
        \ 'params': a:params,
        \ 'on_notification': function('s:on_response', [a:method]),
        \ })
  let l:settled = lsp#utils#_wait(120000, {-> has_key(s:responses, a:method)}, 100)
  call assert_equal(0, l:settled, a:method . ' timed out')
  if l:settled != 0
    return
  endif

  let l:data = remove(s:responses, a:method)
  call assert_true(has_key(l:data, 'response'), a:method . ' produced no response')
  if !has_key(l:data, 'response')
    return
  endif
  let l:response = l:data.response
  call assert_false(lsp#client#is_error(l:response), a:method . ' returned an error')
  call assert_true(has_key(l:response, 'result'), a:method . ' returned no result')
  if has_key(l:response, 'result')
    call assert_equal(a:expected, l:response.result, a:method)
  endif
endfunction

call s:request('textDocument/completion', {
      \ 'position': {'character': 16, 'line': 7},
      \ 'textDocument': {'uri': s:uri},
      \ }, s:expected.completion)
call s:request('textDocument/hover', {
      \ 'position': {'character': 8, 'line': 3},
      \ 'textDocument': {'uri': s:uri},
      \ }, s:expected.hover)
call s:request('textDocument/codeAction', {
      \ 'context': {'diagnostics': []},
      \ 'range': {
      \   'end': {'character': 8, 'line': 7},
      \   'start': {'character': 6, 'line': 7},
      \ },
      \ 'textDocument': {'uri': s:uri},
      \ }, VizeE2EExpectedCodeActions(s:uri))
call s:request('textDocument/formatting', {
      \ 'options': {'insertSpaces': v:true, 'tabSize': 2},
      \ 'textDocument': {'uri': s:uri},
      \ }, s:expected.formatting)
call s:request('textDocument/semanticTokens/full', {
      \ 'textDocument': {'uri': s:uri},
      \ }, s:expected.semantic_tokens)
call s:request('textDocument/rename', {
      \ 'newName': 'quantity',
      \ 'position': {'character': 8, 'line': 3},
      \ 'textDocument': {'uri': s:uri},
      \ }, VizeE2EExpectedRename(s:uri))

call lsp#stop_server('vize')
let s:stopped = lsp#utils#_wait(
      \ 10000,
      \ {-> lsp#get_server_status('vize') ==# 'exited'},
      \ 100,
      \ )
call assert_equal(0, s:stopped, 'the vize language server did not stop')

if !empty(v:errors)
  if $VIZE_E2E_ERROR_PATH !=# ''
    call writefile(v:errors, $VIZE_E2E_ERROR_PATH)
  endif
  cquit 1
endif

quitall!
