<script setup lang="ts">
import { CopyButton } from "./copy-button.ts";
</script>

<art title="Copy Button" status="ready">
  <variant name="Default" default>
    <CopyButton value="https://example.com/invite/team-alpha" />
  </variant>
  <variant name="Custom labels">
    <CopyButton
      value="https://example.com/invite/team-alpha"
      idle-label="Copy invite link"
      copied-label="Invite link copied"
      error-label="Could not copy"
    />
  </variant>
  <variant name="Disabled">
    <CopyButton value="secret" idle-label="Copy token" disabled />
  </variant>
</art>
