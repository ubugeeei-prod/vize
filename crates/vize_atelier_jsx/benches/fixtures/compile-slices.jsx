import { ref } from "vue";
import Card from "./Card";
import List from "./List";

// Constructs the shared L2 lowering admits today.
export const Dashboard = () => (
  <section class="dashboard" id={dashboardId}>
    <header class="topbar">
      <h1>{title}</h1>
      <button type="button" class={tone} onClick={refresh}>
        Refresh
      </button>
    </header>
    <ul class="metrics">
      {metrics.map((metric) => (
        <li key={metric.id} class="metric">
          <span class="label">{metric.label}</span>
          <strong class="value">{metric.value}</strong>
        </li>
      ))}
    </ul>
    {open ? <footer class="footer">{footer}</footer> : <i />}
    <Card title="Totals" {...cardProps}>
      <input v-model={query} />
    </Card>
    <List>{(item) => <b>{item.total}</b>}</List>
  </section>
);

// `&&` over a value and a non-element branch.
export const Status = () => (
  <p class="status">
    {count && <b>{count}</b>}
    {busy ? "Loading" : <i class="idle" />}
  </p>
);

// `_` modifiers on events and directives.
export const Search = () => (
  <form onSubmit_prevent={submit}>
    <input v-model_trim={term} onKeyup_enter={submit} v-focus:start_once={focus} />
  </form>
);

// Mixed text with values, and slot functions that return values.
export const Greeting = () => (
  <Card>
    Hello {name}, you have {unread} new messages.
    <List>{(row) => row.label}</List>
  </Card>
);

// A block-bodied component with setup statements.
export const Labeled = () => {
  const label = ref("labeled");
  return <div class={label.value}>{label.value}</div>;
};
