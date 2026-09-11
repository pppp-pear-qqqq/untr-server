import { Ajax } from '/common/script/ajax.js';
import { sleep } from '/common/script/utils.js';
import { toast } from './util/toast.js';

let portal = 'https://untroche.com';
(window as any).change_portal = function (url: string) {
	portal = url;
};

const login = document.getElementById('login') as HTMLFormElement
const register = document.getElementById('register') as HTMLFormElement
let target: HTMLFormElement | null = null;

if (login) login.addEventListener('submit', (ev) => {
	ev.preventDefault();
	target = login;
	window.open(`${portal}/auth`, '', `popup,width=640,height=140`);
});
if (register) register.addEventListener('submit', (ev) => {
	ev.preventDefault();
	target = register;
	window.open(`${portal}/auth`, '', 'popup,width=640,height=140');
});

window.addEventListener('message', async (ev) => {
	if (ev.origin !== portal) {
		console.log('origin mismatch', ev.origin, portal);
		return;
	}
	if (target == null) {
		console.log('target not found');
		return;
	}
	target.querySelector<HTMLInputElement>('input[name="code"]')!.value = ev.data;
	try {
		const ret = await new Ajax(target).send('text');
		toast.success(`id: ${ret}`);
		await sleep(2000);
		location.href = 'home';
	} catch (err: any) {
		toast.error(err.message);
	}
});

document.getElementById('logout')?.addEventListener('click', async (ev) => {
	ev.preventDefault();
	const href = (ev.currentTarget as HTMLAnchorElement).href;
	try {
		await new Ajax(href).send('text');
		toast.success('ログアウトしました');
		await sleep(2000);
		location.reload();
	} catch (err: any) {
		toast.error(err.message);
	}
});
