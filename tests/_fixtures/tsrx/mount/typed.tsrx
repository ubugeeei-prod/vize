import { ref } from 'vue';

export function Typed({ size, label }: { size: number; label?: string }) @{
	const count = ref<number>(0);
	<button
		class="typed"
		data-size={size as unknown as string}
		onClick={(event: Event) => (count.value += event.type === 'click' ? size : 0)}
	>
		{label!.toUpperCase()}: {(count.value satisfies number) + 1}
		<i>{['a', 'b'].map((x: string): string => x + size).join(',')}</i>
	</button>
}
