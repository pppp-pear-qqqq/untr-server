import { Ajax } from '/common/script/ajax.js';
import { bake, time_formatter as formatter } from '/common/script/utils.js';
import { toast } from './util/toast.js';
import { fav_actors, fav_locations } from './fav.js';

const tabs = document.querySelector<HTMLElement>('#timeline>.tabs')!;
for (const location of fav_locations) {
	tabs.appendChild(bake('label', (e) => {
		e.role = 'button';
		e.append(
			bake('input', (e) => {
				e.type = 'radio';
				e.name = 'tab';
				e.value = location[0];
				e.addEventListener('change', () => {
					reload(e.value);
				})
			}),
			document.createTextNode(location[1]),
		);
	}));
	tabs.appendChild(bake('a', (e) => {
		e.href = `location/${location[0]}`;
		e.textContent = location[1];
	}));
}

const parent = document.getElementById('timeline')!;
const container = document.getElementById('chat_list')!;
const template = document.getElementById(`${container.id}-template`) as HTMLTemplateElement;

async function reload(key: 'actor' | string, quiet: boolean = false) {
	let query = new URLSearchParams();
	if (key === 'actor') for (const actor of fav_actors) {
		query.append('actor', actor);
	} else {
		query.append('location', key);
	}
	if (query) try {
		let ret = await new Ajax('chat').query(query).send('json');
		console.log(ret);
		if (!quiet) toast.success('発言を読み込みました');
		const fragment = document.createDocumentFragment();
		ret.forEach((item: any) => {
			const node = template.content.cloneNode(true) as DocumentFragment;
			(node.firstElementChild as HTMLElement).dataset.id = item.id;
			node.querySelector<HTMLImageElement>('.icon>img')!.src = item.icon;
			node.querySelector('.name')!.textContent = item.name;
			node.querySelector('.id')!.textContent += item.actor;
			node.querySelector('.body')!.innerHTML = item.body;
			const location = node.querySelector<HTMLAnchorElement>('.location')!;
			location.href = `location/${item.location[0]}`;
			location.textContent = item.location[1] ?? '';
			node.querySelector('.timestamp')!.textContent = formatter.format(new Date(item.timestamp * 1000));
			fragment.insertBefore(node, fragment.firstChild);
		});
		container.replaceChildren(fragment);
		parent.scroll({ top: parent.scrollHeight, behavior: 'smooth' });
	} catch (err: any) {
		toast.error(err.message);
	}
}

reload('actor', true);
