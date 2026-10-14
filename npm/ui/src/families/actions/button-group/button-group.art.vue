<script setup lang="ts">
import { ButtonGroup, ButtonGroupItem } from "./button-group.ts";
</script>

<art title="Button Group" status="ready">
  <variant name="Toolbar" default>
    <ButtonGroup role="toolbar" aria-label="Text formatting">
      <ButtonGroupItem value="bold">Bold</ButtonGroupItem>
      <ButtonGroupItem value="italic">Italic</ButtonGroupItem>
      <ButtonGroupItem value="underline">Underline</ButtonGroupItem>
    </ButtonGroup>
  </variant>
  <variant name="Plain group">
    <ButtonGroup aria-label="Pagination">
      <ButtonGroupItem value="previous">Previous</ButtonGroupItem>
      <ButtonGroupItem value="next">Next</ButtonGroupItem>
    </ButtonGroup>
  </variant>
  <variant name="Vertical">
    <ButtonGroup role="toolbar" orientation="vertical" aria-label="Alignment">
      <ButtonGroupItem value="top">Top</ButtonGroupItem>
      <ButtonGroupItem value="middle">Middle</ButtonGroupItem>
      <ButtonGroupItem value="bottom">Bottom</ButtonGroupItem>
    </ButtonGroup>
  </variant>
  <variant name="Disabled item">
    <ButtonGroup role="toolbar" aria-label="Text formatting">
      <ButtonGroupItem value="bold">Bold</ButtonGroupItem>
      <ButtonGroupItem value="strikethrough" disabled>Strikethrough</ButtonGroupItem>
    </ButtonGroup>
  </variant>
  <variant name="Disabled group">
    <ButtonGroup role="toolbar" aria-label="Text formatting" disabled>
      <ButtonGroupItem value="bold">Bold</ButtonGroupItem>
      <ButtonGroupItem value="italic">Italic</ButtonGroupItem>
    </ButtonGroup>
  </variant>
</art>
