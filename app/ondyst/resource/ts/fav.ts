const key = {
	actor: 'fav/actor',
	location: 'fav/location',
};

export const fav_actors = new Set<string>(localStorage.getItem(key.actor)?.split(','));
export const fav_locations = new Set<string>(localStorage.getItem(key.location)?.split(','));

document.querySelectorAll<HTMLInputElement>('.fav>input').forEach((e) => {
	switch (e.name) {
		case 'actor': e.checked = fav_actors.has(e.value); break;
		case 'location': e.checked = fav_locations.has(e.value); break;
	}
	e.addEventListener('change', () => {
		switch (e.name) {
			case 'actor':
				if (e.checked) fav_actors.add(e.value);
				else fav_actors.delete(e.value);
				localStorage.setItem(key.actor, [...fav_actors].join(','));
				break;
			case 'location':
				if (e.checked) fav_locations.add(e.value);
				else fav_locations.delete(e.value);
				localStorage.setItem(key.location, [...fav_locations].join(','));
				break;
		}
	});
})
