import { Ajax } from '/common/script/ajax.js';
import { bake, format_time } from '/common/script/utils.js';
import { toast } from './util/toast.js';
import { fav_actors, fav_locations } from './fav.js';

Ajax.wait = 50;

const tab_bar = document.querySelector<HTMLElement>('#timeline>.tab-bar')!;
for (const location of fav_locations) {
	tab_bar.appendChild(bake('label', (e) => {
		e.classList.add('tab');
		e.role = 'button';
		e.append(
			bake('input', (e) => {
				e.type = 'radio';
				e.name = 'tab';
				e.value = location[0];
			}),
			bake('a', (e) => {
				e.role = 'button';
				e.href = `location/${location[0]}`;
				e.textContent = location[1];
				e.addEventListener('click', (ev) => {
					const radio = (e.previousElementSibling as HTMLInputElement);
					if (!radio.checked) {
						ev.preventDefault();
						radio.click();
					}
				});
			}),
		);
	}));
}
document.querySelectorAll<HTMLInputElement>('.tab>input').forEach((e) => {
	e.addEventListener('change', () => reload(e.value));
});

const container = document.getElementById('chat_list')!;
const template = document.getElementById(`${container.id}-template`) as HTMLTemplateElement;

async function reload(key: 'actor' | string, quiet: boolean = false) {
	let query = new URLSearchParams();
	if (key === 'actor') {
		if (fav_actors.size === 0) {
			toast.warn('キャラクターをお気に入りに登録していません');
			container.replaceChildren();
			return;
		};
		query.append('actor', [...fav_actors].join(','));
	} else {
		query.append('location', key);
	}
	if (query) try {
		let ret = await new Ajax('chat').query(query).send('json');
		if (!quiet) toast.success('発言を読み込みました');
		const fragment = document.createDocumentFragment();
		ret.forEach((item: any) => {
			const node = template.content.cloneNode(true) as DocumentFragment;
			(node.firstElementChild as HTMLElement).dataset.id = item.id;
			node.querySelector('.chat_id')!.textContent += item.id;
			if (item.icon) node.querySelector<HTMLImageElement>('.icon>img')!.src = item.icon;
			node.querySelector('.name')!.textContent = item.name;
			node.querySelector('.id')!.textContent += item.actor;
			node.querySelector('.body')!.innerHTML = item.body;
			const location = node.querySelector<HTMLAnchorElement>('.location')!;
			location.href = `location/${item.location[0]}`;
			location.textContent = item.location[1] ?? '';
			node.querySelector('.timestamp')!.textContent = format_time(item.timestamp);
			fragment.insertBefore(node, fragment.firstChild);
		});
		container.replaceChildren(fragment);
		container.scroll({ top: container.scrollHeight, behavior: 'smooth' });
	} catch (err: any) {
		toast.error(err.message);
	}
}

setTimeout(() => {
	const select_tab = document.querySelector<HTMLInputElement>('[name="tab"]:checked')?.value;
	if (select_tab && (select_tab !== 'actor' || fav_actors.size !== 0)) reload(select_tab, true);
	else console.log('初回読み込みスキップ');
}, 1);
