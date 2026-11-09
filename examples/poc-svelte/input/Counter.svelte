<script>
  let { title } = $props();
  let count = $state(0);
  let name = $state("world");
  let items = $state(["write", "compile", "ship"]);
  let tone = $derived(count > 3 ? "hot" : "cold");

  function increment() {
    count += 1;
  }
</script>

<section class="counter">
  <h1>{title}</h1>
  <button type="button" class={tone} onclick={increment}>
    Clicked {count} times
  </button>
  {#if count > 3}
    <p>Hot!</p>
  {:else}
    <p>Keep clicking</p>
  {/if}
  <input bind:value={name} />
  <p>Hello {name}!</p>
  <ul>
    {#each items as item}
      <li>{item}</li>
    {/each}
  </ul>
</section>

<style>
  .hot {
    color: crimson;
  }
</style>
