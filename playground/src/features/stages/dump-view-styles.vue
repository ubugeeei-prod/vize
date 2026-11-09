<script setup lang="ts">
defineOptions({ name: "DumpViewStyles01" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
/* Stage page listing: projector-legible mono text, a quiet gutter, and one
   ochre wash for the line that is linked to the source. */

.level-dump {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 12px 0;
  font-family: var(--font-mono);
  font-size: 0.875rem;
  line-height: 1.7;
  color: var(--code-foreground);
  background: var(--code-bg);
  border: 1px solid var(--code-border);
  border-radius: 8px;
  tab-size: 2;
}

.level-line {
  display: flex;
  min-width: max-content;
  padding-inline-end: 20px;
  border-left: 3px solid transparent;
}

.level-line.linkable {
  cursor: pointer;
}

.level-line.linkable:hover {
  background: var(--bg-tertiary);
}

.level-line.linked {
  border-left-color: var(--color-notice);
  background: var(--color-notice-bg);
}

.level-line.selected,
.level-line.selected:hover {
  border-left-color: var(--color-notice);
  background: var(--color-notice-bg);
  box-shadow: inset 0 0 0 1px var(--color-notice-border);
}

.level-line.level-section {
  margin-top: 4px;
}

/* L3 partition overlay: teal static, ochre dynamic. */
.level-mark {
  flex-shrink: 0;
  align-self: center;
  width: 8px;
  height: 8px;
  margin-inline-start: 10px;
  border-radius: 50%;
}

.level-mark.static {
  background: var(--color-teal);
}

.level-mark.dynamic {
  background: var(--color-notice);
}

.level-ln {
  flex-shrink: 0;
  width: 3.5em;
  padding-inline-end: 1.25em;
  text-align: right;
  color: var(--code-line-number);
  opacity: 0.7;
  user-select: none;
}

.level-code {
  white-space: pre;
}

/* Tokens */
.tk-section {
  color: var(--hl-type);
  font-weight: 600;
}

.tk-key {
  color: var(--hl-property);
}

.tk-mnemonic {
  color: var(--hl-keyword);
  font-weight: 600;
}

.tk-tag {
  color: var(--hl-tag);
}

.tk-string {
  color: var(--hl-string);
}

.tk-span {
  color: var(--code-line-number);
}

.tk-number {
  color: var(--hl-number);
}

.tk-call {
  color: var(--hl-function);
}

.tk-absent,
.tk-punct {
  color: var(--hl-delimiter);
}

.tk-attr {
  color: var(--hl-property);
}

.tk-directive {
  color: var(--hl-directive);
}

.tk-mustache {
  color: var(--hl-function);
}

.tk-static,
.tk-dynamic {
  padding: 0 6px;
  border-radius: 4px;
  font-weight: 600;
}

.tk-static {
  color: var(--color-teal);
  background: var(--color-teal-bg);
}

.tk-dynamic {
  color: var(--color-notice);
  background: var(--color-notice-bg);
}

/* Page difference against the previous page of the same stage */
.level-diff,
.level-remarks {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.level-diff-summary {
  margin: 0;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.level-diff-mark {
  flex-shrink: 0;
  width: 1.5em;
  color: var(--text-muted);
  user-select: none;
}

.level-line.diff-add {
  background: var(--color-success-bg);
}

.level-line.diff-add {
  & .level-diff-mark {
    color: var(--color-success);
  }
}

.level-line.diff-remove {
  background: var(--color-error-bg);
}

.level-line.diff-remove {
  & .level-diff-mark {
    color: var(--color-error);
  }
}

/* Optimization remarks */
.level-remarks-empty {
  max-width: 64ch;
  padding: 20px 22px;
  border: 1px dashed var(--border-primary);
  border-radius: 8px;
}

.level-remarks-title {
  margin: 0 0 6px;
  font-size: 0.9375rem;
  font-weight: 600;
}

.level-remark-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0;
  padding: 0;
  overflow: auto;
  list-style: none;
}

.level-remark {
  display: grid;
  grid-template-columns: auto auto 1fr auto;
  align-items: baseline;
  gap: 10px;
  padding: 8px 12px;
  font-size: 0.875rem;
  border: 1px solid var(--border-primary);
  border-radius: 8px;
}

.level-remark-outcome {
  padding: 1px 6px;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  border-radius: 4px;
}

.level-remark.applied {
  & .level-remark-outcome {
    color: var(--color-teal);
    background: var(--color-teal-bg);
  }
}

.level-remark.missed {
  & .level-remark-outcome {
    color: var(--color-notice);
    background: var(--color-notice-bg);
  }
}

.level-remark.analysis {
  & .level-remark-outcome {
    color: var(--text-secondary);
    background: var(--bg-tertiary);
  }
}

.level-remark-pass {
  font-family: var(--font-mono);
  color: var(--text-secondary);
}

.level-remark-message {
  display: flex;
  flex-wrap: wrap;
  gap: 4px 8px;
  align-items: baseline;
}

.level-remark-arg {
  font-family: var(--font-mono);
  font-size: 0.75rem;
  color: var(--text-muted);
}
</style>
