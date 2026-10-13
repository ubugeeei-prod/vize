<script setup lang="ts">
import { BackToTop } from "./back-to-top.ts";
</script>

<art title="Back To Top" status="ready">
  <variant name="Default" default>
    <BackToTop />
  </variant>
  <variant name="Custom label and threshold">
    <BackToTop aria-label="Return to the top of the article" :threshold="120" />
  </variant>
</art>
