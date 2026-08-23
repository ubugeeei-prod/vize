local M = {}

local function write_file(path, contents)
  local handle = assert(io.open(path, "w"))
  handle:write(contents)
  handle:close()
end

local function assert_component_contract_hover(ctx, bufnr, uri, position, expected_hover, label)
  local result = ctx.request(ctx.client, bufnr, "textDocument/hover", {
    position = position,
    textDocument = { uri = uri },
  })
  ctx.assert_eq(result, expected_hover, label)
  assert(
    not result.contents.value:match("__vizeComponentMarker")
      and not result.contents.value:match("__vizeRawProps")
      and not result.contents.value:match("__VizeComponentConstructor"),
    label .. " leaked generated component carrier types: " .. result.contents.value
  )
end

function M.run(ctx)
  local child_path = ctx.workspace_path .. "/src/ContractChild.vue"
  local host_path = ctx.workspace_path .. "/src/ContractHost.vue"
  write_file(child_path, ctx.expected.component_contract_child_source)
  write_file(host_path, ctx.expected.component_contract_host_source)
  vim.cmd("edit " .. vim.fn.fnameescape(host_path))

  local host_bufnr = vim.api.nvim_get_current_buf()
  ctx.assert_eq(vim.bo[host_bufnr].filetype, "vue", "ContractHost fixture filetype")
  assert(vim.lsp.buf_attach_client(host_bufnr, ctx.client.id), "ContractHost buffer did not attach")

  local host_uri = vim.uri_from_bufnr(host_bufnr)
  local settled = vim.wait(180000, function()
    return ctx.published[host_uri] ~= nil
  end, 100)
  if not settled then
    ctx.fail("real server did not publish ContractHost diagnostics", ctx.published)
  end
  ctx.assert_eq(ctx.published[host_uri], {}, "ContractHost diagnostics")

  local hovers = ctx.expected.component_contract_hovers
  assert_component_contract_hover(
    ctx,
    host_bufnr,
    host_uri,
    { character = 8, line = 1 },
    hovers.import_binding,
    "component contract import hover"
  )
  assert_component_contract_hover(
    ctx,
    host_bufnr,
    host_uri,
    { character = 1, line = 3 },
    hovers.script_usage,
    "component contract script usage hover"
  )

  vim.api.nvim_set_current_buf(ctx.scenario_bufnr)
end

return M
