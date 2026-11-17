import { ref } from 'vue';

function Child(props: { text: string }) {
	return <b class="child">{props.text}</b>;
}

export function Plain({ label, count }: { label: string; count: number }) {
	const upper = label.toUpperCase();
	return (
		<p class="plain" data-count={count}>
			{upper}: <Child text={label} />
		</p>
	);
}

export const Arrow = ({ count }: { count: number }) => <span data-count={count}>{count * 2}</span>;

export const Grow = (props: { items: string[] }) => {
	const size = ref(1);
	return (
		<div>
			<button onClick={() => (size.value += 1)}>grow</button>
			<p>{props.items.slice(0, size.value).join(',')}</p>
		</div>
	);
};
