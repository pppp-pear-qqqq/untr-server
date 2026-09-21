import { Ajax } from '/common/script/ajax.js';
import { Pagination } from '/common/script/pagination.js';
import { format_time } from '/common/script/utils.js';
import { toast } from './util/toast.js';
import { Stream } from './util/stream.js';
import init, { to_html } from '../pkg/ondyst.js';

// 要素取得
const key = document.getElementById('location')!.dataset.key;

const container = document.getElementById('chat_list')!;
const template = document.getElementById(`${container.id}-template`) as HTMLTemplateElement;
const size = Number(document.querySelector('.pagination>.size')!.textContent);

const reload = document.querySelector<HTMLElement>('.pagination .reload')!;
const form = document.querySelector<HTMLFormElement>('#post form');

// 再読み込み関連
const page = new Pagination({ size: size, limit_default: 20, limit_max: 100 });
page.callback = (list: any[]) => {
	reload.classList.remove('active');
	const fragment = document.createDocumentFragment();
	list.forEach((item: any) => {
		const node = template.content.cloneNode(true) as DocumentFragment;
		(node.firstElementChild as HTMLElement).dataset.id = item.id;
		const icon = node.querySelector<HTMLAnchorElement>('.icon')!;
		icon.href = `actor/${item.actor}`;
		if (item.icon) (icon.firstElementChild as HTMLImageElement).src = item.icon;
		node.querySelector('.name')!.textContent = item.name;
		node.querySelector('.id')!.textContent += item.actor;
		node.querySelector('.body')!.innerHTML = item.body;
		node.querySelector('.location')!.textContent = item.location[1] ?? '';
		node.querySelector('.timestamp')!.textContent = format_time(item.timestamp);
		fragment.insertBefore(node, fragment.firstChild);
	});
	container.replaceChildren(fragment);
	setting_reply_buttons();
	requestAnimationFrame(() => requestAnimationFrame(() => container.scroll({ top: container.scrollHeight, behavior: 'smooth' })));
};
page.error = (e) => toast.error(e.message);

// 返信ボタンイベント設定
function setting_reply_buttons() {
	if (!form_body) return;
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

// サーバーとのストリーム接続
const stream_option = localStorage.getItem('stream');
if (stream_option && stream_option !== 'off') {
	var stream = new Stream(`/location/${key}/stream`);
	console.log('stream接続開始');
	switch (stream_option) {
		case 'reload': stream.onmessage = function () {
			console.log('update');
			if (new URLSearchParams(location.search).get('offset') == null) page.reload();
		}; break;
		case 'notice': stream.onmessage = function () {
			console.log('update');
			stream.ignore(0);
			reload.classList.add('active');
			toast.info('新しい発言があります');
		}; break;
	}
	stream.onerror = function (error) {
		console.error(error);
	};
}

if (form) {
	var form_body = form?.children.namedItem('body') as HTMLTextAreaElement;
	var preview = document.getElementById('preview') as HTMLDetailsElement;
	// 発言
	let force_submit = false;
	form.addEventListener('submit', async (ev) => {
		ev.preventDefault();
		if (!force_submit && form_body.value.trim() === '') {
			toast.warn('発言内容が空欄です\nこのまま投稿する場合は再度送信してください');
			force_submit = true;
			return;
		}
		try {
			if(stream) stream.ignore(1000);
			await new Ajax(form).send();
			force_submit = false;
			preview.open = false;
			form_body.value = '';
			toast.success('発言しました');
			page.reload();
		} catch (err: any) {
			toast.error(err.message);
		}
	});

	// アイテム使用
	const items = document.getElementById('items');
	if (items) {
		// 使用準備
		items.querySelectorAll<HTMLButtonElement>('[name="message"]').forEach((e) => {
			e.addEventListener('click', () => {
				if (!e.value) {
					toast.warn('使用時テキストがありません（未実装）');
					return;
				}
				form_body.value = `${form_body.value}${e.value}`;
				form_body.focus();
				form_body.setSelectionRange(form_body.value.length, form_body.value.length);
				form_body.dispatchEvent(new Event('input'));
				form_body.dispatchEvent(new Event('change'));
				toast.info('使用時テキストを読み込みました');
			});
		});
		// 使用
		items.addEventListener('submit', async (ev) => {
			try {
				if(stream) stream.ignore(1000);
				await new Ajax(ev.currentTarget as HTMLFormElement).send();
				force_submit = false;
				preview.open = false;
				form_body.value = '';
				toast.success('アイテムを使用しました');
				page.reload();
			} catch (err: any) {
				toast.error(err.message);
			}
		})
	}

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

	// ページ離脱時発言内容保存
	window.addEventListener('beforeunload', () => {
		sessionStorage.setItem('chat/body', form_body.value);
	})
	// 発言内容初期設定
	form_body.value = sessionStorage.getItem('chat/body') ?? '';
	if (location.hash) {
		form_body.value = `${decodeURI(location.hash.slice(1))}\n${form_body.value}`;
		form_body.focus();
		form_body.setSelectionRange(form_body.value.length, form_body.value.length);
	}
}

// 返信ボタンの設定
setting_reply_buttons();

// スクロール
container.scroll({ top: container.scrollHeight, behavior: 'smooth' });

// プレビュー
if (preview!) {
	await init();
	const update_preview = (ev: Event) => {
		if (preview.open) {
			const t = ev.target;
			if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement) switch (t.name) {
				case 'name': preview.querySelector('.name')!.textContent = t.value; break;
				case 'body': preview.querySelector('.body')!.innerHTML = to_html(form_body.value, false); break;
				case 'icon': preview.querySelector<HTMLImageElement>('.icon>img')!.src = t.value; break;
			} else if (t instanceof HTMLDetailsElement) {
				preview.querySelector('.name')!.textContent = form!.querySelector<HTMLInputElement>('input[name="name"]')!.value;
				preview.querySelector('.body')!.innerHTML = to_html(form_body.value, false);
				preview.querySelector<HTMLImageElement>('.icon>img')!.src = form!.querySelector<HTMLInputElement>('input[name="icon"]')!.value;
			}
		}
	}
	// form.addEventListener('input', update_preview);
	form!.addEventListener('change', update_preview);	// 流石にinputで処理は重そう
	preview.addEventListener('toggle', update_preview);
}
