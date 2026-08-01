function! s:detect_art_vue() abort
  setlocal filetype=art-vue
  if empty(&l:syntax)
    setlocal syntax=vue
  endif
endfunction

augroup vize_filetypes
  autocmd!
  autocmd BufNewFile,BufRead *.vue setlocal filetype=vue
  autocmd BufNewFile,BufRead *.art.vue call <SID>detect_art_vue()
augroup END
