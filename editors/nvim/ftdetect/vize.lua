vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
  group = vim.api.nvim_create_augroup("vize_filetypes", { clear = true }),
  pattern = "*.vue",
  callback = function()
    vim.bo.filetype = "vue"
  end,
})

local function has_explicit_art_vue_language()
  local language = vim.treesitter.language.get_lang("art-vue")
  if language ~= "art-vue" then
    return true
  end

  -- get_filetypes() starts with the language itself; a second match records
  -- an explicit identity registration for the Art Vue filetype.
  local registrations = 0
  for _, filetype in ipairs(vim.treesitter.language.get_filetypes(language)) do
    if filetype == "art-vue" then
      registrations = registrations + 1
    end
  end
  return registrations > 1
end

if not has_explicit_art_vue_language() and not vim.treesitter.language.add("art-vue") then
  vim.treesitter.language.register("vue", "art-vue")
end

vim.api.nvim_create_autocmd({ "BufNewFile", "BufRead" }, {
  group = vim.api.nvim_create_augroup("vize_art_vue_filetypes", { clear = true }),
  pattern = "*.art.vue",
  callback = function()
    vim.bo.filetype = "art-vue"
    if vim.bo.syntax == "" then
      vim.bo.syntax = "vue"
    end
  end,
})
