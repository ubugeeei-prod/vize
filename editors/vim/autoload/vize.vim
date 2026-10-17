let s:feature_keys = [
      \ 'enabled',
      \ 'nativeLinkedEditing',
      \ 'diagnostics',
      \ 'lint',
      \ 'typecheck',
      \ 'editor',
      \ 'ecosystem',
      \ 'optionsApi',
      \ 'legacyVue2',
      \ 'completion',
      \ 'signatureHelp',
      \ 'hover',
      \ 'definition',
      \ 'references',
      \ 'documentSymbols',
      \ 'workspaceSymbols',
      \ 'codeActions',
      \ 'rename',
      \ 'formatting',
      \ 'codeLens',
      \ 'semanticTokens',
      \ 'documentLinks',
      \ 'foldingRanges',
      \ 'inlayHints',
      \ 'fileRename',
      \ 'autoInsert',
      \ 'corsa',
      \ 'tsgo',
      \ 'crossFile',
      \ ]

function! s:exclusive_profile(lint_only) abort
  let l:options = {}
  for l:key in s:feature_keys
    let l:options[l:key] = v:false
  endfor
  let l:options.enabled = a:lint_only
  let l:options.lint = a:lint_only
  return l:options
endfunction

let s:profiles = {
      \ 'lint': s:exclusive_profile(v:true),
      \ 'off': s:exclusive_profile(v:false),
      \ 'recommended': {
      \   'editor': v:true,
      \   'ecosystem': v:true,
      \   'lint': v:true,
      \   'typecheck': v:true,
      \ },
      \ }

let s:default_config = {
      \ 'allowlist': ['vue', 'art-vue'],
      \ 'cmd': ['vize', 'lsp'],
      \ 'initialization_options': {},
      \ }

function! vize#profile(name) abort
  if !has_key(s:profiles, a:name)
    throw 'unknown vize profile: ' . string(a:name)
  endif
  return deepcopy(s:profiles[a:name])
endfunction

function! vize#default_config() abort
  return deepcopy(s:default_config)
endfunction

function! vize#normalize(...) abort
  let l:opts = a:0 ? a:1 : {}
  if type(l:opts) != v:t_dict
    throw 'vize.setup options must be a dictionary'
  endif

  let l:config = vize#default_config()
  if has_key(l:opts, 'profile')
    let l:config.initialization_options = vize#profile(l:opts.profile)
  endif

  if has_key(l:opts, 'cmd')
    let l:config.cmd = s:assert_string_list('cmd', l:opts.cmd)
  endif

  if has_key(l:opts, 'allowlist')
    let l:config.allowlist = s:assert_string_list('allowlist', l:opts.allowlist)
  endif

  if has_key(l:opts, 'initialization_options')
    if type(l:opts.initialization_options) != v:t_dict
      throw 'vize.initialization_options must be a dictionary'
    endif
    let l:config.initialization_options = deepcopy(l:opts.initialization_options)
  endif

  return l:config
endfunction

function! vize#vim_lsp_config(...) abort
  let l:config = vize#normalize(a:0 ? a:1 : {})
  return {
        \ 'name': 'vize',
        \ 'cmd': function('s:server_cmd', [l:config.cmd]),
        \ 'allowlist': l:config.allowlist,
        \ 'initialization_options': l:config.initialization_options,
        \ }
endfunction

function! vize#setup(...) abort
  let l:config = vize#normalize(a:0 ? a:1 : {})
  " vim-lsp defines its public functions through autoload, so checking only
  " exists('*lsp#register_server') misses the normal startup order where its
  " plugin commands are loaded before an autoload function is first called.
  if exists('*lsp#register_server') || exists(':LspStatus') == 2
    call lsp#register_server(vize#vim_lsp_config(l:config))
  else
    echohl WarningMsg
    echom 'vim-lsp not found; install vim-lsp and call vize#setup() again'
    echohl None
  endif
  return l:config
endfunction

" vim-lsp does not advertise dynamic file watching. Forward saved project
" configuration/helper buffers through its actual notification transport.
function! vize#notify_config_saved(file) abort
  if !exists('*lsp#get_server_status') || lsp#get_server_status('vize') !=# 'running'
    return 0
  endif
  if a:file !~? '\.\%(jsonc\?\|[jt]sx\?\|[cm][jt]s\)$'
    return 0
  endif
  let l:root_uri = lsp#get_server_root_uri('vize')
  if l:root_uri !~# '^file:'
    return 0
  endif
  let l:root = s:physical_path(lsp#utils#uri_to_path(l:root_uri))
  let l:file = s:physical_path(a:file)
  if stridx(l:file, substitute(l:root, '/\+$', '', '') . '/') != 0
    return 0
  endif
  call lsp#callbag#pipe(
        \ lsp#notification('vize', {
        \   'method': 'workspace/didChangeWatchedFiles',
        \   'params': {'changes': [{'uri': lsp#utils#path_to_uri(fnamemodify(a:file, ':p')), 'type': 2}]},
        \ }),
        \ lsp#callbag#subscribe({}),
        \ )
  return 1
endfunction

function! s:physical_path(path) abort
  let l:path = substitute(resolve(fnamemodify(a:path, ':p')), '\\', '/', 'g')
  return has('win32') || has('win64') ? tolower(l:path) : l:path
endfunction

function! s:assert_string_list(name, value) abort
  if type(a:value) != v:t_list || empty(a:value)
    throw 'vize.' . a:name . ' must be a non-empty list'
  endif

  for l:item in a:value
    if type(l:item) != v:t_string || empty(l:item)
      throw 'vize.' . a:name . ' must contain non-empty strings'
    endif
  endfor

  return deepcopy(a:value)
endfunction

function! s:server_cmd(cmd, server_info) abort
  return deepcopy(a:cmd)
endfunction
