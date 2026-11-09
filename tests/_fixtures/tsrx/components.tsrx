function Badge({ text }: { text: string }) @{
	<span class="badge">{text}</span>
}

function Section({ as, title }: { as: 'section' | 'article'; title: string }) @{
	<{as} class="panel">
		<h2>{title}</h2>
		<Badge text={title} />
	</{as}>
}

export function App({ title }: { title: string }) @{
	const Body = title.length > 3 ? Section : Badge;

	<main>
		<Section as="article" title={title} />
		<{Body} as="section" title={title} text={title} />
	</main>
}
