export function Field({ name, value, disabled, tone }: {
	name: string;
	value: string;
	disabled: boolean;
	tone: string;
}) @{
	const id = `field-${name}`;

	<div class={tone} data-name={name}>
		<label for={id}>{name}</label>
		<input id={id} type="text" value={value} disabled={disabled} />
		<output aria-live="polite" title={value.length > 3 ? 'long' : undefined}>{value}</output>
	</div>
}
