import { Ajax } from '/common/script/ajax.js';

const ev_change = new Event('change', { bubbles: true });

// 一括設定
const bulk = document.getElementById('bulk') as HTMLFormElement;
const drop_area = bulk.querySelector<HTMLElement>('.drop-area')!;
const files = bulk.querySelector<HTMLInputElement>('[name="files"]')!;
drop_area.addEventListener('drop', ev => {
	const data = ev.dataTransfer?.files;
	drop_area.classList.remove('file-hover');
	if (data) {
		ev.preventDefault();
		files.files = data;
		files.dispatchEvent(ev_change);
	}
})
drop_area.addEventListener('dragover', ev => {
	ev.preventDefault();
	if (ev.dataTransfer) ev.dataTransfer.dropEffect = 'move';
})
drop_area.addEventListener('dragenter', ev => {
	if (ev.dataTransfer?.files) drop_area.classList.add('file-hover');
})
drop_area.addEventListener('dragleave', () => drop_area.classList.remove('file-hover'))
files.addEventListener('change', () => bulk.requestSubmit())
bulk.addEventListener('submit', ev => {
	// ev.preventDefault();
	// console.log('submit');
})

// 詳細設定
const setting = document.getElementById('setting') as HTMLFormElement;
setting.addEventListener('submit', ev => {
	ev.preventDefault();
	const path = setting.querySelector<HTMLInputElement>('[name="path"]')?.value;
	if (path == null || path == '') return;
	const action = `${setting.action}/${path}`;
	let fetch_dest = setting.querySelector<HTMLInputElement>('[name="fetch-dest"]')?.value ?? null;
	if (fetch_dest === 'none') fetch_dest = null;
	const ty = (ev.submitter as HTMLButtonElement).value;
	let files: File[];
	let config;
	switch (ty) {
		case 'file': {
			const e = setting.querySelector('file')!;
			const file = e.querySelector<HTMLInputElement>('[name="file"]')?.files?.[0];
			if (!file) {
				return;
			}
			files = [file];
			config = { type: "File" };
		} break;
		case 'random-file': {
			const e = setting.querySelector('random-file')!;
			files = Array.from(e.querySelectorAll<HTMLInputElement>('[name="file"]')).map(e => e.files?.item(0) ?? null).filter(f => f != null);
			const weights = Array.from(e.querySelectorAll<HTMLInputElement>('[name="weight"]')).map(e => Number(e.value));
			if (files.length !== weights.length) {
				return;
			}
			const cache = Boolean(e.querySelector<HTMLInputElement>('[name="cache"]')?.value ?? true);
			config = { type: "RandomFile", weights, cache };
		} break;
		case 'font-render': {
			const e = setting.querySelector('font-render')!;
			const file = e.querySelector<HTMLInputElement>('[name="file"]')?.files?.[0];
			if (!file) {
				return;
			}
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

	new Ajax(action).method('POST').body(form_data, 'form').send('json').then((res: string[]) => {
		console.log(res);
	})
});
