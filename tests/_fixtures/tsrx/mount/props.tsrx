const FALLBACK = 'fallback';

export function Defaults({ label = 'none', count = 1, flag = false, tags = ['a'], format = (n: number) => `#${n}`, fallback = FALLBACK }: { label?: string; count?: number; flag?: boolean; tags?: string[]; format?: (n: number) => string; fallback?: string }) @{
	<p data-flag={String(flag)}>{label}: {format(count)} {tags.join(',')} {fallback}</p>
}

export function Renamed({ label: text, count: n = 2 }: { label: string; count?: number }) @{
	const doubled = n * 2;
	<p title={text}>{text}: {n} {doubled} {String({ text }.text)}</p>
}

export function Rest({ label, ...rest }: { label: string }) @{
	<p {...rest}>{label}</p>
}
