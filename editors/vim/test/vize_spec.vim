set runtimepath^=editors/vim
runtime autoload/vize.vim

let s:defaults = vize#normalize({})
call assert_equal(['vize', 'lsp'], s:defaults.cmd)
call assert_equal(['vue', 'art-vue'], s:defaults.allowlist)
call assert_equal({
      \ 'editor': v:true,
      \ 'ecosystem': v:true,
      \ 'lint': v:true,
      \ 'typecheck': v:true,
      \ }, s:defaults.initialization_options)

let s:recommended = vize#normalize({'profile': 'recommended'})
call assert_equal({
      \ 'editor': v:true,
      \ 'ecosystem': v:true,
      \ 'lint': v:true,
      \ 'typecheck': v:true,
      \ }, s:recommended.initialization_options)

call assert_equal({'lint': v:true}, vize#profile('lint'))
call assert_equal({}, vize#profile('off'))

let s:lint_profile = vize#profile('lint')
let s:lint_profile.lint = v:false
call assert_equal({'lint': v:true}, vize#profile('lint'))

let s:default_copy = vize#default_config()
let s:default_copy.cmd[0] = 'changed'
call assert_equal(['vize', 'lsp'], vize#default_config().cmd)

let s:custom = vize#normalize({
      \ 'cmd': ['/tmp/vize', 'lsp', '--debug'],
      \ 'allowlist': ['vue'],
      \ 'initialization_options': {'hover': v:true, 'references': v:true},
      \ })
call assert_equal(['/tmp/vize', 'lsp', '--debug'], s:custom.cmd)
call assert_equal(['vue'], s:custom.allowlist)
call assert_equal({'hover': v:true, 'references': v:true}, s:custom.initialization_options)

let s:mutable_opts = {'initialization_options': {'hover': v:true}}
let s:mutable_config = vize#normalize(s:mutable_opts)
let s:mutable_opts.initialization_options.hover = v:false
call assert_equal({'hover': v:true}, s:mutable_config.initialization_options)

let s:lsp_config = vize#vim_lsp_config({'profile': 'off'})
call assert_equal('vize', s:lsp_config.name)
call assert_equal(['vue', 'art-vue'], s:lsp_config.allowlist)
call assert_equal({}, s:lsp_config.initialization_options)
call assert_equal(['vize', 'lsp'], s:lsp_config.cmd({}))

try
  call vize#profile('missing')
  call assert_report('expected unknown profile to fail')
catch /unknown vize profile/
endtry

try
  call vize#normalize({'cmd': []})
  call assert_report('expected empty cmd to fail')
catch /cmd/
endtry

try
  call vize#normalize({'allowlist': ['']})
  call assert_report('expected empty allowlist item to fail')
catch /allowlist/
endtry

runtime ftdetect/vize.vim
execute 'edit ' . fnameescape(tempname() . '.vue')
call assert_equal('vue', &filetype)
execute 'edit ' . fnameescape(tempname() . '.art.vue')
call assert_equal('art-vue', &filetype)

if !empty(v:errors)
  cquit
endif

quitall!
