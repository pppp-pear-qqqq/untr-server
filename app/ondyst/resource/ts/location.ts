import { Ajax } from '/common/script/ajax.js';
import { toast } from './import.js';

const form = document.querySelector<HTMLFormElement>('#post form')!;
form.addEventListener('submit', (ev) => {
	ev.preventDefault();
	new Ajax(form).send().then(() => {
		toast.success('発言しました');
		reload();
	}).catch((err) => {
		toast.error(err.message);
	});
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

const formatter = new Intl.DateTimeFormat('ja-JP', {
	year: 'numeric', month: '2-digit', day: '2-digit',
	hour: '2-digit', minute: '2-digit', second: '2-digit',
	hour12: false
});

function reload(offset?: number, limit?: number) {
	const params = new URLSearchParams();
	if (offset != null) params.set('offset', offset.toString());
	if (limit != null) params.set('limit', limit.toString());
	new Ajax(window.location.href).get(params).send('json').then(ret => {
		const container = document.getElementById('chat_list')!;
		const template = document.getElementById(`${container.id}-template`) as HTMLTemplateElement;
		const fragment = document.createDocumentFragment();
		ret.forEach((item: any) => {
			const node = template.content.cloneNode(true) as DocumentFragment;
			(node.firstElementChild as HTMLElement).dataset.id = item.id;
			node.querySelector<HTMLImageElement>('.icon>img')!.src = item.icon;
			node.querySelector('.name')!.textContent = item.name;
			node.querySelector('.id')!.textContent += item.actor;
			node.querySelector('.body')!.textContent = item.body;
			node.querySelector('.timestamp')!.textContent = formatter.format(new Date(item.timestamp * 1000));
			fragment.appendChild(node);
		});
		container.replaceChildren(fragment);
	})
}
