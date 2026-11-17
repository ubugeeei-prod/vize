import { ref } from 'vue';

function Child(props: { label: string }) @{
	<span class="child">{props.label}</span>
}

export function Host({ tag }: { tag: string }) @{
	<{tag} class="host" data-tag={tag}>
		{'dynamic '}
		<b>{tag}</b>
	</{tag}>
}

export function Toggle() @{
	const tag = ref<'section' | 'article'>('section');
	<>
		<{tag.value} class="toggle">{tag.value}</{tag.value}>
		<button
			onClick={() => {
				tag.value = tag.value === 'section' ? 'article' : 'section';
			}}
		>
			toggle
		</button>
	</>
}

export function Pick({ label }: { label: string }) @{
	const component = Child;
	<{component} label={label} />
}
