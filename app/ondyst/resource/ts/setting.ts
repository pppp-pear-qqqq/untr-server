import { Ajax } from '/common/script/ajax.js';
import { bake } from '/common/script/utils.js';
import { toast } from './util/toast.js';

let data: Record<string, string> = {};
const form = document.querySelector('form')!;
form.addEventListener('change', (ev) => {
	const e = ev.target as HTMLInputElement | HTMLTextAreaElement;
	e.classList.add('changed');
	data[e.name] = e.value;
});
form.addEventListener('submit', async (ev) => {
	ev.preventDefault();
	if (Object.keys(data).length === 0) {
		toast.warn('更新内容がありません');
		return;
	}
	try {
		await new Ajax(form.action).method('PATCH').body(data, 'json').send();
		toast.success('更新しました');
	} catch (err: any) {
		toast.error(err.message);
	}
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

// ハンドアウト
const handout = document.getElementById('handout') as HTMLDataListElement;
const ho_title = form.querySelector<HTMLInputElement>('[name="ho_title"]')!;
const ho_body = form.querySelector<HTMLInputElement>('[name="ho_body"]')!;
document.querySelector<HTMLButtonElement>('[name="handout-reroll"]')!.addEventListener('click', () => {
	const ho = handout.options[Math.floor(Math.random() * handout.options.length)];
	ho_title.value = ho.textContent;
	ho_body.value = ho.value;
	ho_title.dispatchEvent(new Event('change', { bubbles: true }));
	ho_body.dispatchEvent(new Event('change', { bubbles: true }));
});
document.querySelector<HTMLButtonElement>('[name="handout-reset"]')!.addEventListener('click', () => {
	ho_title.value = ho_title.dataset.init!;
	ho_body.value = ho_body.dataset.init!;
	ho_title.classList.remove('changed');
	ho_body.classList.remove('changed');
	delete data[ho_title.name];
	delete data[ho_body.name];
});

// ローカル設定
const stream = document.querySelector<HTMLSelectElement>('select[name="stream"]')!;
const stream_mode = localStorage.getItem('stream');
if (stream_mode) {
	stream.value = stream_mode;
} else {
	stream.value = 'off';
}
stream.addEventListener('change', () => {
	if (stream.value === 'off') localStorage.removeItem('stream');
	else localStorage.setItem('stream', stream.value);
});
