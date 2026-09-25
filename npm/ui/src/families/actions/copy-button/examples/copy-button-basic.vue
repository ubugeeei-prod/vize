<!-- Copies an invite link to the clipboard and announces the copied state in its label. -->
<script setup lang="ts">
import { ref, useId } from "vue";

import { CopyButton } from "../copy-button.ts";

const inviteLink = "https://example.com/invite/team-alpha";
const linkId = useId();
const lastCopied = ref("");
</script>

<template>
  <div>
    <code :id="linkId">{{ inviteLink }}</code>
    <CopyButton
      :value="inviteLink"
      idle-label="Copy invite link"
      copied-label="Invite link copied"
      error-label="Could not copy"
      :aria-describedby="linkId"
      @copy="(value) => (lastCopied = value)"
    />
    <p aria-live="polite">{{ lastCopied ? "Link is on your clipboard." : "" }}</p>
  </div>
</template>
