if exists('g:loaded_lode')
  finish
endif
let g:loaded_lode = 1

lua require('lode').setup()
