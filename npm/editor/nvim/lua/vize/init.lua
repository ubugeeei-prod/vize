local config = require("vize.config")

local M = {
  config = config,
}

function M.setup(opts)
  local resolved = config.normalize(opts)

  assert(vim.lsp ~= nil and vim.lsp.config ~= nil, "vize.nvim requires Neovim 0.11+")
  vim.lsp.config("vize", resolved)

  if resolved.autostart and vim.lsp.enable ~= nil then
    vim.lsp.enable("vize")
  end

  return resolved
end

return M
