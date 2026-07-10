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

let s:override_profile = vize#normalize({
      \ 'profile': 'lint',
      \ 'initialization_options': {'hover': v:true, 'references': v:true},
      \ })
call assert_equal({'hover': v:true, 'references': v:true}, s:override_profile.initialization_options)

let s:mutable_opts = {'initialization_options': {'hover': v:true}}
let s:mutable_config = vize#normalize(s:mutable_opts)
let s:mutable_opts.initialization_options.hover = v:false
call assert_equal({'hover': v:true}, s:mutable_config.initialization_options)

let s:mutable_nested_opts = {'initialization_options': {'nested': {'hover': v:true}}}
let s:mutable_nested_config = vize#normalize(s:mutable_nested_opts)
let s:mutable_nested_opts.initialization_options.nested.hover = v:false
call assert_equal({'nested': {'hover': v:true}}, s:mutable_nested_config.initialization_options)

let s:mutable_cmd_opts = {'cmd': ['vize', 'lsp']}
let s:mutable_cmd_config = vize#normalize(s:mutable_cmd_opts)
let s:mutable_cmd_opts.cmd[0] = 'changed'
call assert_equal(['vize', 'lsp'], s:mutable_cmd_config.cmd)

let s:mutable_allowlist_opts = {'allowlist': ['vue', 'art-vue']}
let s:mutable_allowlist_config = vize#normalize(s:mutable_allowlist_opts)
let s:mutable_allowlist_opts.allowlist[0] = 'changed'
call assert_equal(['vue', 'art-vue'], s:mutable_allowlist_config.allowlist)

let s:default_config_nested_copy = vize#default_config()
let s:default_config_nested_copy.initialization_options.editor = v:false
call assert_equal({
      \ 'editor': v:true,
      \ 'ecosystem': v:true,
      \ 'lint': v:true,
      \ 'typecheck': v:true,
      \ }, vize#default_config().initialization_options)

let s:lsp_config = vize#vim_lsp_config({'profile': 'off'})
call assert_equal('vize', s:lsp_config.name)
call assert_equal(['vue', 'art-vue'], s:lsp_config.allowlist)
call assert_equal({}, s:lsp_config.initialization_options)
call assert_equal(['vize', 'lsp'], s:lsp_config.cmd({}))
let s:lsp_cmd_copy = s:lsp_config.cmd({})
let s:lsp_cmd_copy[0] = 'changed'
call assert_equal(['vize', 'lsp'], s:lsp_config.cmd({}))

let s:custom_lsp_config = vize#vim_lsp_config({
      \ 'cmd': ['/tmp/vize', 'lsp'],
      \ 'allowlist': ['art-vue'],
      \ 'initialization_options': {'hover': v:true},
      \ })
call assert_equal(['/tmp/vize', 'lsp'], s:custom_lsp_config.cmd({}))
call assert_equal(['art-vue'], s:custom_lsp_config.allowlist)
call assert_equal({'hover': v:true}, s:custom_lsp_config.initialization_options)

try
  call vize#profile('missing')
  call assert_report('expected unknown profile to fail')
catch /unknown vize profile/
endtry

try
  call vize#normalize([])
  call assert_report('expected non-dictionary options to fail')
catch /dictionary/
endtry

try
  call vize#normalize({'cmd': []})
  call assert_report('expected empty cmd to fail')
catch /cmd/
endtry

try
  call vize#normalize({'cmd': ['vize', 1]})
  call assert_report('expected non-string cmd item to fail')
catch /cmd/
endtry

try
  call vize#normalize({'allowlist': ['']})
  call assert_report('expected empty allowlist item to fail')
catch /allowlist/
endtry

try
  call vize#normalize({'allowlist': 'vue'})
  call assert_report('expected non-list allowlist to fail')
catch /allowlist/
endtry

try
  call vize#normalize({'initialization_options': []})
  call assert_report('expected non-dictionary initialization options to fail')
catch /initialization_options/
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
