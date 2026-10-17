if exists('g:loaded_vize')
  finish
endif

let g:loaded_vize = 1

command! VizeSetup call vize#setup()

augroup vize_saved_project_config
  autocmd!
  autocmd BufWritePost *.json,*.jsonc,*.js,*.mjs,*.cjs,*.ts,*.mts,*.cts,*.jsx,*.tsx call vize#notify_config_saved(expand('<afile>:p'))
augroup END
