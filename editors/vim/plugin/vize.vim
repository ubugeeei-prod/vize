if exists('g:loaded_vize')
  finish
endif

let g:loaded_vize = 1

command! VizeSetup call vize#setup()
