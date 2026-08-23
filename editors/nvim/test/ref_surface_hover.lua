local M = {}

local function write_file(path, contents)
  local handle = assert(io.open(path, "w"))
  handle:write(contents)
  handle:close()
end

local function assert_ref_surface_hover(ctx, bufnr, uri, position, expected_hover, label)
  local result = ctx.request(ctx.client, bufnr, "textDocument/hover", {
    position = position,
    textDocument = { uri = uri },
  })
  ctx.assert_eq(result, expected_hover, label)
  assert(
    not result.contents.value:match("Ref<unknown>")
      and not result.contents.value:match("ComputedRef<unknown>")
      and not result.contents.value:match("MaybeRef<unknown>"),
    label .. " degraded to an unknown reactive type: " .. result.contents.value
  )
end

function M.run(ctx)
  local ref_path = ctx.workspace_path .. "/src/RefSurface.vue"
  write_file(ref_path, ctx.expected.ref_surface_source)
  vim.cmd("edit " .. vim.fn.fnameescape(ref_path))

  local ref_bufnr = vim.api.nvim_get_current_buf()
  ctx.assert_eq(vim.bo[ref_bufnr].filetype, "vue", "RefSurface fixture filetype")
  assert(vim.lsp.buf_attach_client(ref_bufnr, ctx.client.id), "RefSurface buffer did not attach")

  local ref_uri = vim.uri_from_bufnr(ref_bufnr)
  local settled = vim.wait(180000, function()
    return ctx.published[ref_uri] ~= nil
  end, 100)
  if not settled then
    ctx.fail("real server did not publish RefSurface diagnostics", ctx.published)
  end
  ctx.assert_eq(ctx.published[ref_uri], {}, "RefSurface diagnostics")

  local hovers = ctx.expected.ref_surface_hovers
  assert_ref_surface_hover(
    ctx,
    ref_bufnr,
    ref_uri,
    { character = 8, line = 3 },
    hovers.script_count,
    "script ref hover"
  )
  assert_ref_surface_hover(
    ctx,
    ref_bufnr,
    ref_uri,
    { character = 8, line = 4 },
    hovers.script_doubled,
    "script computed hover"
  )
  assert_ref_surface_hover(
    ctx,
    ref_bufnr,
    ref_uri,
    { character = 8, line = 5 },
    hovers.script_button,
    "script template-ref hover"
  )
  assert_ref_surface_hover(
    ctx,
    ref_bufnr,
    ref_uri,
    { character = 28, line = 9 },
    hovers.template_count,
    "template ref hover"
  )
  assert_ref_surface_hover(
    ctx,
    ref_bufnr,
    ref_uri,
    { character = 40, line = 9 },
    hovers.template_doubled,
    "template computed hover"
  )
  assert_ref_surface_hover(
    ctx,
    ref_bufnr,
    ref_uri,
    { character = 54, line = 9 },
    hovers.template_button,
    "template template-ref hover"
  )

  vim.api.nvim_set_current_buf(ctx.scenario_bufnr)
end

return M
