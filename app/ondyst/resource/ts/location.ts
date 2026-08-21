import { Ajax } from '/common/script/ajax.js';
import { bake } from '/common/script/funcs.js';
import { toast } from './import.js';

const form = document.querySelector<HTMLFormElement>('#post form')!;
form.addEventListener('submit', (ev) => {
	ev.preventDefault();
	new Ajax(form).send().then(() => {
		toast.success('発言しました');
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
