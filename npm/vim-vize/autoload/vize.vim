let s:profiles = {
      \ 'lint': {'lint': v:true},
      \ 'off': {},
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
      \ 'initialization_options': s:profiles.lint,
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
  if exists('*lsp#register_server')
    call lsp#register_server(vize#vim_lsp_config(l:config))
  else
    echohl WarningMsg
    echom 'vim-lsp not found; install vim-lsp and call vize#setup() again'
    echohl None
  endif
  return l:config
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
