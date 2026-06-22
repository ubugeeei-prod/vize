if vim.g.loaded_vize == 1 then
  return
end

vim.g.loaded_vize = 1

vim.api.nvim_create_user_command("VizeSetup", function()
  require("vize").setup()
end, {
  desc = "Configure the Vize language server",
})
