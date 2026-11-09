<script setup lang="ts">
defineOptions({ name: "StageRailStyles01" });
</script>

<!-- These global selectors theme generated and third-party DOM owned by the feature. -->
<!-- eslint-disable-next-line vue/require-scoped-style -->
<style>
/* The stage rail (stations joined by the pounce line) and the pass timeline. */

/* Stage rail */
.level-rail {
  display: flex;
  align-items: stretch;
  padding: 18px 20px 14px;
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
}

.level-station {
  display: grid;
  grid-template-columns: auto 1fr;
  grid-template-rows: auto auto;
  column-gap: 10px;
  align-items: baseline;
  padding: 8px 12px;
  min-width: 0;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 10px;
  color: var(--text-secondary);
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.level-station:hover {
  background: var(--bg-secondary);
}

.level-station:focus-visible,
.level-step:focus-visible,
.level-subtab:focus-visible,
.level-line:focus-visible {
  outline: 2px solid var(--dv-ochre);
  outline-offset: 2px;
}

.level-station.active {
  background: var(--bg-secondary);
  border-color: var(--border-primary);
  color: var(--text-primary);
}

.level-station.empty {
  opacity: 0.55;
}

.level-station-ordinal {
  grid-row: 1 / span 2;
  align-self: center;
  display: inline-grid;
  place-items: center;
  width: 34px;
  height: 34px;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  font-weight: 700;
  border: 1.5px solid currentColor;
  border-radius: 50%;
}

.level-station.active {
  & .level-station-ordinal {
    background: var(--text-primary);
    border-color: var(--text-primary);
    color: var(--bg-primary);
  }
}

.level-station-name {
  font-size: 1rem;
  font-weight: 600;
  white-space: nowrap;
}

.level-station-facts {
  font-size: 0.75rem;
  color: var(--text-muted);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.level-pounce {
  flex: 1;
  min-width: 16px;
  align-self: center;
  height: 4px;
  margin: 0 2px;
  background-image: radial-gradient(circle, var(--text-muted) 1.1px, transparent 1.6px);
  background-size: 7px 4px;
  background-repeat: repeat-x;
  background-position: center;
}

/* Pass timeline */
.level-timeline {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-primary);
  flex-shrink: 0;
}

.level-timeline-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

/* Where the time went, in run order: one segment per measured step. */
.level-time-strip {
  display: flex;
  flex: 1;
  gap: 2px;
  height: 6px;
}

.level-time-segment {
  min-width: 3px;
  border-radius: 3px;
  opacity: 0.55;
}

.level-time-segment.current {
  opacity: 1;
}

.level-time-segment.rung-l1 {
  background: var(--hl-tag);
}

.level-time-segment.rung-l2 {
  background: var(--hl-directive);
}

.level-time-segment.rung-l3 {
  background: var(--color-notice);
}

.level-steps {
  display: flex;
  flex-wrap: wrap;
  align-items: flex-start;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
  flex: 1;
  min-width: 0;
}

/* One pass-manager walk: its passes above a bracket naming the walk. */
.level-walk {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.level-walk-steps {
  display: flex;
  gap: 6px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.level-walk-label {
  display: flex;
  gap: 8px;
  padding: 3px 10px 0;
  border-top: 1.5px solid var(--border-primary);
  font-family: var(--font-mono);
  font-size: 0.6875rem;
  color: var(--text-muted);
  white-space: nowrap;
}

.level-walk.fusable {
  & .level-walk-label {
    border-top-color: var(--hl-directive);
  }
}

/* Fixed like the step times, so a live walk time never reflows the row. */
.level-walk-time {
  min-width: 6ch;
  margin-inline-start: auto;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.level-step {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 5px 11px 5px 9px;
  font-family: var(--font-mono);
  font-size: 0.8125rem;
  background: transparent;
  border: 1px solid var(--border-primary);
  border-radius: 999px;
  color: var(--text-secondary);
  cursor: pointer;
}

.level-step:hover {
  background: var(--bg-secondary);
}

.level-step.current {
  border-color: var(--text-primary);
  color: var(--text-primary);
  background: var(--bg-secondary);
}

.level-step-mark {
  width: 9px;
  height: 9px;
  border: 1.5px solid var(--dv-ochre);
  border-radius: 50%;
}

.level-step.changed {
  & .level-step-mark {
    background: var(--dv-ochre);
  }
}

.level-step.producer {
  & .level-step-mark {
    border-color: var(--text-primary);
    background: var(--text-primary);
    border-radius: 2px;
  }
}

.level-step-rung,
.level-step-time {
  font-size: 0.6875rem;
  color: var(--text-muted);
}

/* A fixed slot so live timings never reflow the steps while typing. */
.level-step-time {
  min-width: 6ch;
  text-align: right;
  font-variant-numeric: tabular-nums;
}

.level-step-remarks {
  min-width: 18px;
  padding: 0 5px;
  font-size: 0.6875rem;
  line-height: 16px;
  text-align: center;
  color: var(--color-notice);
  background: var(--color-notice-bg);
  border-radius: 8px;
}

.level-timeline-summary {
  margin: 0 0 0 auto;
  font-size: 0.75rem;
  color: var(--text-muted);
  white-space: nowrap;
}
</style>
