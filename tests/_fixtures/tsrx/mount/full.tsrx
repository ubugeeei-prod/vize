export function Spread(props: Record<string, unknown>) @{
	<div {...props} />
}

export function Merged(props: { kind: string }) @{
	<p class={props.kind} {...props}>{props.kind}</p>
}

export function Alias(props: { title: string }) @{
	const p = props;
	<p>{p.title}: {JSON.stringify(p)}</p>
}
