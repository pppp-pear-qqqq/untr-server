import { Ajax } from '/common/script/ajax.js';
import { bake, time_formatter as formatter } from '/common/script/utils.js';
import { toast } from './toast.js';
import { fav_actors, fav_locations } from './fav.js';

const tabs = document.querySelector<HTMLElement>('#timeline>.tabs')!;
for (const location of fav_locations) {
	const item = location.split(':');
	tabs.appendChild(bake('label', (e) => {
		e.classList.add('tab');
		e.role = 'button';
		e.append(
			bake('input', (e) => {
				e.type = 'radio';
				e.name = 'tab';
				e.value = item[0];
				e.addEventListener('change', () => {
					if (e.checked) reload(e.value);
				})
			}),
			document.createTextNode(item[1]),
		);
	}));
}

const parent = document.getElementById('timeline')!;
const container = document.getElementById('chat_list')!;
const template = document.getElementById(`${container.id}-template`) as HTMLTemplateElement;

async function reload(key: 'actor' | string, quiet: boolean = false) {
	const target = key === 'actor' ? [...fav_actors].join('_') : key;
	try {
		let ret: any[];
		if (target) {
			ret = await new Ajax(`location/${target}`).send('json');
		} else {
			ret = [];
		}
		if (!quiet) toast.success('発言を読み込みました');
		const fragment = document.createDocumentFragment();
		ret.forEach((item) => {
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
		parent.scroll({ top: parent.scrollHeight, behavior: 'smooth' });
	} catch (err: any) {
		toast.error(err.message);
	}
}

reload('actor', true);
