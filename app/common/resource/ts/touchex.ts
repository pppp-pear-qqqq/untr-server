let tap_flag = false;
let longtap_handle: number | null = null;
export function addTapEvent(e: HTMLElement, tap_time = 50, longtap_time = 480) {
	e.addEventListener('touchstart', ev => {
		tap_flag = true;
		setTimeout(() => tap_flag = false, tap_time);
		longtap_handle = setTimeout(() => {
			// これ本来はちゃんとロングタップ開始時点のデータを見なきゃいけないので、どうにかしてください
			e.dispatchEvent(new TouchEvent('longtap', {
				touches: [...ev.touches],
				targetTouches: [...ev.targetTouches],
				changedTouches: [...ev.changedTouches],
				ctrlKey: ev.ctrlKey,
				shiftKey: ev.shiftKey,
				altKey: ev.altKey,
				metaKey: ev.metaKey,
			}));
			longtap_handle = null;
		}, longtap_time);
	});
	e.addEventListener('touchmove', () => {
		tap_flag = false;
		if (longtap_handle !== null) {
			clearTimeout(longtap_handle);
			longtap_handle = null;
		}
	});
	e.addEventListener('touchend', ev => {
		if (tap_flag) {
			e.dispatchEvent(new TouchEvent('tap', {
				touches: [...ev.touches],
				targetTouches: [...ev.targetTouches],
				changedTouches: [...ev.changedTouches],
				ctrlKey: ev.ctrlKey,
				shiftKey: ev.shiftKey,
				altKey: ev.altKey,
				metaKey: ev.metaKey,
			}));
			tap_flag = false;
		}
		if (longtap_handle !== null) {
			clearTimeout(longtap_handle);
			longtap_handle = null;
		}
	});
}
