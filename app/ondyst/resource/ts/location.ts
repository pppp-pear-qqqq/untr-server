import { Ajax } from '/common/script/ajax.js';
import { time_formatter as formatter } from '/common/script/utils.js';
import { toast } from './toast.js';

// 発言
const form = document.querySelector<HTMLFormElement>('#post form')!;
form.addEventListener('submit', async (ev) => {
	ev.preventDefault();
	try {
		await new Ajax(form).send();
		(form.children.namedItem('body') as HTMLTextAreaElement).value = '';
		toast.success('発言しました');
		reload({ scroll: 'bottom' });
	} catch (err: any) {
		toast.error(err.message);
	}
});

// アイコン選択
const icon_dialog = form.querySelector<HTMLDialogElement>('dialog')!;
const icon_button = form.querySelector<HTMLButtonElement>('button.icon')!;
const icon = icon_button.firstElementChild as HTMLImageElement;
icon_button.addEventListener('click', () => {
	icon_dialog.showModal();
});
icon_dialog.querySelectorAll<HTMLInputElement>('input').forEach((e) => {
	e.addEventListener('click', () => {
		icon.src = e.value;
		icon_dialog.close();
	});
});

// 再読み込み
const parent = document.getElementById('timeline')!;
const container = document.getElementById('chat_list')!;
const template = document.getElementById(`${container.id}-template`) as HTMLTemplateElement;

let offset_max: number = Number.MAX_SAFE_INTEGER;
const limit_default = 20;
const limit_max = 100;

if (container.childElementCount < limit_default) offset_max = Number(new URLSearchParams(location.search).get('offset') ?? 0);

async function reload({ offset = 0, limit = limit_default, scroll = 'none' }: { offset?: number; limit?: number; scroll?: 'top' | 'bottom' | 'none' }) {
	const o = Math.min(offset, offset_max);
	const l = Math.min(limit, limit_max);
	const params = new URLSearchParams();
	params.set('offset', o.toString());
	params.set('limit', l.toString());
	try {
		const ret = await new Ajax(window.location.pathname).get(params).send('json');
		window.history.pushState(null, '', `${location.pathname}?offset=${o}&limit=${l}`);
		const fragment = document.createDocumentFragment();
		ret.forEach((item: any) => {
			const node = template.content.cloneNode(true) as DocumentFragment;
			(node.firstElementChild as HTMLElement).dataset.id = item.id;
			node.querySelector<HTMLImageElement>('.icon>img')!.src = item.icon;
			node.querySelector('.name')!.textContent = item.name;
			node.querySelector('.id')!.textContent += item.actor;
			node.querySelector('.body')!.textContent = item.body;
			node.querySelector('.timestamp')!.textContent = formatter.format(new Date(item.timestamp * 1000));
			fragment.insertBefore(node, fragment.firstChild);
		});
		container.replaceChildren(fragment);
		if (container.childElementCount < l) offset_max = o;
		if (scroll === 'none') return;
		parent.scroll({ top: scroll === 'top' ? 0 : parent.scrollHeight, behavior: 'smooth' });
	} catch (err: any) {
		toast.error(err.message);
	}
}

// ページネーション
document.querySelectorAll<HTMLElement>('[data-page]').forEach(e => {
	e.addEventListener('click', () => {
		const page = Number(e.dataset.page);
		const search = new URLSearchParams(location.search);
		const offset = Number(search.get('offset') ?? 0);
		const limit = Number(search.get('limit') ?? limit_default);
		reload({ offset: Math.max(offset + page * limit, 0), limit: limit, scroll: 'top' });
	});
})

parent.scroll({ top: parent.scrollHeight, behavior: 'smooth' });
