import { ref } from 'vue';

function Label(props: { text: string; count: number }) @{
	<span class="label">{props.text}: {props.count}</span>
}

export function Counter({ title, step }: { title: string; step: number }) @{
	const count = ref(0);
	const heading = title.toUpperCase();

	<section>
		<h1>{heading}</h1>
		<Label text={title} count={count.value} />
		<button onClick={() => (count.value += step)}>add</button>
		@if (count.value > 2) {
			<p class="many">many</p>
		} @else if (count.value > 0) {
			<p class="some">some</p>
		} @else {
			<p class="none">none</p>
		}
	</section>
}
