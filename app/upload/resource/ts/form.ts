import { Ajax } from '/common/script/ajax.js';

const setting = document.getElementById('setting') as HTMLFormElement;
setting.addEventListener('submit', ev => {
	ev.preventDefault();
	const path = setting.querySelector<HTMLInputElement>('[name="path"]')?.value;
	if (path == null || path == '') return;
	setting.action = `control/${path}`;
	let fetch_dest = setting.querySelector<HTMLInputElement>('[name="fetch-dest"]')?.value ?? null;
	if (fetch_dest === 'none') fetch_dest = null;
	const ty = (ev.submitter as HTMLButtonElement).value;
	let files: File[];
	let config;
	switch (ty) {
		case 'file': {
			const e = setting.querySelector('file')!;
			const file = e.querySelector<HTMLInputElement>('[name="file"]')?.files?.[0];
			if (!file) return;
			files = [file];
			config = { type: "File" };
		} break;
		case 'random-file': {
			const e = setting.querySelector('random-file')!;
			files = Array.from(e.querySelectorAll<HTMLInputElement>('[name="file"]')).flatMap(e => Array.from(e.files ?? []));
			const weights = Array.from(e.querySelectorAll<HTMLInputElement>('[name="weight"]')).map(e => Number(e.value));
			const cache = Boolean(e.querySelector<HTMLInputElement>('[name="cache"]')?.value ?? true);
			config = { type: "RandomFile", weights, cache };
		} break;
		case 'font-render': {
			const e = setting.querySelector('font-render')!;
			const file = e.querySelector<HTMLInputElement>('[name="file"]')?.files?.[0];
			if (!file) return;
			files = [file];
			const width = Number(e.querySelector<HTMLInputElement>('[name="width"]')?.value ?? 48);
			const height = Number(e.querySelector<HTMLInputElement>('[name="height"]')?.value ?? 48);
			const line_length = Number(e.querySelector<HTMLInputElement>('[name="line_length"]')?.value ?? 18);
			config = { type: "FontRender", width, height, line_length };
		} break;
		default: return;
	}

	const form_data = new FormData();
	if (fetch_dest) form_data.append('fetch_dest', fetch_dest);
	for (const file of files) form_data.append('files', file);
	form_data.append('config', JSON.stringify(config));

	new Ajax(setting.action).method('POST').body(form_data, 'form').send('json').then((res: string[]) => {
		console.log(res);
	})
});
