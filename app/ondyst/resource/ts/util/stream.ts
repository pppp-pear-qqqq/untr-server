export class Stream {
	private event_source: EventSource;
	private _ignore: boolean = false;
	private auto_ignore: number = 1000;

	constructor(url: string, auto_ignore: number = 1000) {
		this.event_source = new EventSource(url);
		this.auto_ignore = auto_ignore;
	}
	set onmessage(callback: (event: MessageEvent) => void) {
		this.event_source.onmessage = (event: MessageEvent) => {
			if (this._ignore) return;
			this.ignore(this.auto_ignore);
			callback(event);
		};
	}
	set onerror(callback: (event: Event) => void) {
		this.event_source.onerror = callback;
	}

	ignore(ms: number) {
		this._ignore = true;
		if (ms > 0) setTimeout(() => this._ignore = false, ms);
	}
}
