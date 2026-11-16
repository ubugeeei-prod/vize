import { useSlots } from 'vue';

function Card() @{
	const slots = useSlots();
	<>{slots.default?.()}</>
}

export function Panel({ title, count }: { title: string; count: number }) @{
	<Card>
		<h2 class="title">{title}</h2>
		<p>count: {count}</p>
	</Card>
}
