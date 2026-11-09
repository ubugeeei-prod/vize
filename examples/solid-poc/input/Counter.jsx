import { createSignal, Show } from "solid-js";

export function Counter(props) {
  const [count, setCount] = createSignal(props.start);
  return (
    <section class="counter">
      <h1>{props.title}</h1>
      <button
        type="button"
        class={count() > 3 ? "hot" : "cold"}
        onClick={() => setCount(count() + 1)}
      >
        Clicked {count()} times
      </button>
      <Show when={count() > 3}>
        <p>Hot!</p>
      </Show>
    </section>
  );
}
