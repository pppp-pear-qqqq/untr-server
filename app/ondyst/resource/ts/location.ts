import { Ajax } from '/common/script/ajax.js';
import { Pagination } from '/common/script/pagination.js';
import { time_formatter as formatter } from '/common/script/utils.js';
import { toast } from './toast.js';

// 要素取得
const parent = document.getElementById('timeline')!;
const container = document.getElementById('chat_list')!;
const template = document.getElementById(`${container.id}-template`) as HTMLTemplateElement;
const size = Number(document.querySelector('.pagination>.size')!.textContent);

// 再読み込み関連
const page = new Pagination({ size: size, limit_default: 20, limit_max: 100 });
page.callback = (list: any[]) => {
	const fragment = document.createDocumentFragment();
	list.forEach((item: any) => {
		const node = template.content.cloneNode(true) as DocumentFragment;
		(node.firstElementChild as HTMLElement).dataset.id = item.id;
		node.querySelector<HTMLImageElement>('.icon>img')!.src = item.icon;
		node.querySelector('.name')!.textContent = item.name;
		node.querySelector('.id')!.textContent += item.actor;
		node.querySelector('.body')!.innerHTML = item.body;
		node.querySelector('.timestamp')!.textContent = formatter.format(new Date(item.timestamp * 1000));
		fragment.insertBefore(node, fragment.firstChild);
	});
	container.replaceChildren(fragment);
	setting_reply_buttons();
	parent.scroll({ top: parent.scrollHeight, behavior: 'smooth' });
};
page.error = (e) => toast.error(e.message);

// 発言
const form = document.querySelector<HTMLFormElement>('#post form')!;
const form_name = form.children.namedItem('name') as HTMLInputElement;
const form_body = form.children.namedItem('body') as HTMLTextAreaElement;

let force_submit = false;
form.addEventListener('submit', async (ev) => {
	ev.preventDefault();
	if (!force_submit && form_body.value.trim() === '') {
		toast.warn('発言内容が空欄です\nこのまま投稿する場合は再度送信してください');
		force_submit = true;
		return;
	}
	try {
		await new Ajax(form).send();
		form_body.value = '';
		force_submit = false;
		toast.success('発言しました');
		page.reload();
	} catch (err: any) {
		toast.error(err.message);
	}
});

// アイコン選択
const icon_dialog = form.querySelector<HTMLDialogElement>('dialog');
if (icon_dialog) {
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
}

// 発言内容初期設定
window.addEventListener('beforeunload', () => {
	localStorage.setItem('chat/body', form_body.value);
	if (form_name) localStorage.setItem('chat/name', form_name.value);
	else localStorage.removeItem('chat/name');
	const icon = form.querySelector<HTMLInputElement>('input[name="icon"]:checked')?.value;
	if (icon) localStorage.setItem('chat/icon', icon);
	else localStorage.removeItem('chat/icon');
})
if (location.hash) {
	const id = Number(location.hash.slice(3));
	if (id) {
		if (location.hash.startsWith('#a-')) form_body.value = `>>${id}\n`;
		else if (location.hash.startsWith('#m-')) form_body.value = `@${id}\n`;
	}
}
form_body.value += localStorage.getItem('chat/body') ?? '';
const save_name = localStorage.getItem('chat/name');
if (save_name) (form.children.namedItem('name') as HTMLInputElement).value = save_name;
const save_icon = localStorage.getItem('chat/icon');
if (save_icon) {
	const target = form.querySelector<HTMLInputElement>(`input[name="icon"][value="${save_icon}"]`);
	if (target) {
		target.click();
		form.querySelector<HTMLImageElement>('button.icon>img')!.src = save_icon;
	}
}

// 返信ボタンイベント設定
function setting_reply_buttons() {
	document.querySelectorAll<HTMLElement>('button.reply').forEach(e => {
		e.addEventListener('click', () => {
			const id = e.closest<HTMLElement>('[data-id]')!.dataset.id;
			if (form_body.value.startsWith('>>')) {
				form_body.value = `>>${id} ${form_body.value}`;
			} else {
				form_body.value = `>>${id}\n${form_body.value}`;
			}
		});
	});
}

setting_reply_buttons();

parent.scroll({ top: parent.scrollHeight, behavior: 'smooth' });
