customElements.define('check-img', class extends HTMLElement {
	static formAssociated = true;	// フォーム連携
	static observedAttributes = ['off-src', 'off-alt', 'on-src', 'on-alt', 'size'];

	private internals;
	private checked;
	private off;
	private on;

	constructor() {
		super();
		this.internals = this.attachInternals();
		this.checked = false;
		this.off = document.createElement('img');
		this.on = document.createElement('img');
	}

	connectedCallback() {
		this.role = 'checkbox';
		this.tabIndex = Number(this.getAttribute('tabIndex'));
		this.updateFormValue();

		this.addEventListener('click', this.toggle.bind(this));
		this.addEventListener('keydown', ev => {
			if (ev.key === ' ' || ev.key === 'Enter') {
				ev.preventDefault();
				this.toggle();
			}
		});

		const shadow = this.attachShadow({ mode: 'closed' });

		const style = document.createElement('style');
		style.textContent = 'img{vertical-align:bottom}:host([checked]) .off{display:none}:host(:not([checked])) .on{display:none}';

		this.off.className = 'off';
		this.off.alt = '無効';
		this.on.className = 'on';
		this.on.alt = '有効';

		shadow.appendChild(style);
		shadow.appendChild(this.off);
		shadow.appendChild(this.on);
	}

	attributeChangedCallback(name: string, _old_value: string, new_value: string) {
		switch (name) {
			case 'off-src': this.off.src = new_value; break;
			case 'off-alt': this.off.alt = new_value; break;
			case 'on-src': this.on.src = new_value; break;
			case 'on-alt': this.on.alt = new_value; break;
			case 'size': {
				const size = Number(new_value);
				this.off.width = size;
				this.off.height = size;
				this.on.width = size;
				this.on.height = size;
			} break;
		}
	}

	toggle() {
		this.checked = !this.checked;
		this.setAttribute('aria-checked', String(this.checked));
		this.toggleAttribute('checked', this.checked);
		this.updateFormValue();
	}

	private updateFormValue() {
		if (this.getAttribute('name')) this.internals.setFormValue(this.checked ? 'on' : '');
	}

	// public API
	get form() {
		return this.internals.form;
	}
	get name() {
		return this.getAttribute('name') ?? '';
	}
	get type() {
		return 'checkbox';
	}
	get checkedValue() {
		return this.checked;
	}
});

customElements.define('hamburger-menu', class extends HTMLElement {
	constructor() {
		super();
	}
	connectedCallback() {
		this.role = 'button';
		this.tabIndex = Number(this.getAttribute('tabIndex'));

		this.addEventListener('click', this.toggle.bind(this));
		this.addEventListener('keydown', ev => {
			if (ev.key === ' ' || ev.key === 'Enter') {
				ev.preventDefault();
				this.toggle();
			}
		});

		const shadow = this.attachShadow({ mode: 'closed' });

		const style = document.createElement('link');
		style.rel = 'stylesheet';
		style.href = 'common/style/hamburger.css';
		// const style = document.createElement('style');
		// style.textContent = ':host{--siz: 36px;--wht: calc(var(--siz) / 9);--pad: calc(var(--siz) / 25);width:var(--siz);height:var(--siz);padding:var(--pad) 0!important;color:var(--border-color);display:flex;box-sizing:border-box;flex-direction:column;justify-content:space-between;@supports (x:attr(size px,36px)){--siz: attr(size px, 36px)}}div{height:var(--wht);border-radius:var(--wht);background-color:currentColor;transition:transform .3s}:host([open]) div{&:nth-of-type(1){transform:translateY(calc((var(--siz) - var(--wht)) / 2 - var(--pad))) rotate(45deg)}&:nth-of-type(2){transform:translate(75%) scaleX(0)}&:nth-of-type(3){transform:translateY(calc((var(--siz) - var(--wht)) / -2 + var(--pad))) rotate(-45deg)}}';
		shadow.appendChild(style);

		shadow.appendChild(document.createElement('div'));
		shadow.appendChild(document.createElement('div'));
		shadow.appendChild(document.createElement('div'));
	}

	toggle() {
		this.toggleAttribute('open', this.getAttribute('open') === null);
	}
});
