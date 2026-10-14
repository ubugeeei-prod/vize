<script setup lang="ts">
import { Toolbar, ToolbarItem } from "./toolbar.ts";
</script>

<art title="Toolbar" status="ready">
  <variant name="Default" default>
    <Toolbar aria-label="Document actions">
      <ToolbarItem value="save">Save</ToolbarItem>
      <ToolbarItem value="share">Share</ToolbarItem>
      <ToolbarItem value="print">Print</ToolbarItem>
    </Toolbar>
  </variant>
  <variant name="Vertical">
    <Toolbar aria-label="Drawing tools" orientation="vertical">
      <ToolbarItem value="pen">Pen</ToolbarItem>
      <ToolbarItem value="eraser">Eraser</ToolbarItem>
    </Toolbar>
  </variant>
  <variant name="Right to left">
    <Toolbar aria-label="Document actions" dir="rtl">
      <ToolbarItem value="save">Save</ToolbarItem>
      <ToolbarItem value="share">Share</ToolbarItem>
    </Toolbar>
  </variant>
  <variant name="Disabled">
    <Toolbar aria-label="Document actions" disabled>
      <ToolbarItem value="save">Save</ToolbarItem>
      <ToolbarItem value="share">Share</ToolbarItem>
    </Toolbar>
  </variant>
</art>
