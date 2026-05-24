vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
  group = vim.api.nvim_create_augroup("vize_filetypes", { clear = true }),
  pattern = "*.vue",
  callback = function()
    vim.bo.filetype = "vue"
  end,
})

vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
  group = vim.api.nvim_create_augroup("vize_art_vue_filetypes", { clear = true }),
  pattern = "*.art.vue",
  callback = function()
    vim.bo.filetype = "art-vue"
  end,
})
