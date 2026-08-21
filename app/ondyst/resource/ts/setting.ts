import { Ajax } from '/common/script/ajax.js';
import { bake } from '/common/script/funcs.js';
import { toast } from './import.js';

let data: Record<string, string> = {};
const form = document.querySelector('form')!;
form.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>('[name]').forEach(e => {
	e.addEventListener('change', () => {
		e.classList.add('changed');
		data[e.name] = e.value;
	})
});
form.addEventListener('submit', async (ev) => {
	ev.preventDefault();
	const form = ev.currentTarget as HTMLFormElement;
	new Ajax(form.action).method('PATCH').body(data, 'json').send().then(() => {
		toast.success('更新しました');
	}).catch(err => {
		toast.error(err.message);
	});
});

const icon_list = document.getElementById('icon_list') as HTMLTextAreaElement;
const icon_preview = icon_list.nextElementSibling as HTMLElement;
icon_list.addEventListener('change', () => {
	icon_preview.replaceChildren();
	icon_list.value.split('\n').forEach(line => {
		const trimmed = line.trim();
		if (trimmed) icon_preview.appendChild(bake('img', e => {
			e.src = trimmed;
			e.width = 48;
		}));
	});
});
const portrait_list = document.getElementById('portrait_list') as HTMLTextAreaElement;
const portrait_preview = portrait_list.nextElementSibling as HTMLElement;
portrait_list.addEventListener('change', () => {
	portrait_preview.replaceChildren();
	portrait_list.value.split('\n').forEach(line => {
		const trimmed = line.trim();
		if (trimmed) portrait_preview.appendChild(bake('img', e => {
			e.src = trimmed;
			e.width = 384;
		}));
	});
});
