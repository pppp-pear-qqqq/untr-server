import { Toast } from '/common/script/notice.js';

export const success = { icon: 'ri-checkbox-circle-line', color: '#3dc763c0' };
export const info = { icon: 'ri-information-line', color: '#63a4f9c0' };
export const warn = { icon: 'ri-spam-line', color: '#d3cd16c0' };
export const error = { icon: 'ri-spam-line', color: '#ed3d3dc0', duration: -1 };

Toast.types = {
	success: { ...success },
	info: { ...info },
	warn: { ...warn },
	error: { ...error },
};

export const toast = new Toast({}, {});

export async function sleep(ms: number) {
	return new Promise(resolve => setTimeout(resolve, ms));
}
