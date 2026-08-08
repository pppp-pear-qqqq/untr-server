import { Ajax } from '/common/script/ajax.js';

const portal = 'http://localhost:8080';

const register = document.getElementById('register') as HTMLFormElement
register.addEventListener('submit', ev => {
	ev.preventDefault();
	window.open(`${portal}/auth`, '', 'popup');
});
window.addEventListener('message', ev => {
	if (ev.origin !== portal) return;
	register.querySelector<HTMLInputElement>('input[name="code"]')!.value = ev.data;
	console.log(ev.data);
	new Ajax(register).send('text').then(ret => {
		console.log(ret);
	});
});
