import { getModeOsPrefers, setModeCurrent, setModeUserPrefers, type ToastSettings } from '@skeletonlabs/skeleton';
import { type BaseResult, InfoType, Theme, type Toast } from '$lib/types';
import { invoke } from '@tauri-apps/api';
import { toast } from '$lib/stores';


// This function is used to generate toastSettings from a toast object
export function generateToast(toast: Toast): ToastSettings {
	let classes = 'variant-filled-secondary';
	switch (toast.type) {
		case InfoType.ERROR:
			classes = 'variant-filled-error';
			break;
		case InfoType.SUCCESS:
			classes = 'variant-filled-success';
			break;
		case InfoType.WARNING:
			classes = 'variant-filled-warning';
			break;
		default:
			break;
	}

	return {
		message: toast.message,
		timeout: toast.timeout,
		hideDismiss: true,
		classes
	} as ToastSettings;
}

// This function is used to set the theme for the aoo with the help of the skeleton library
export function setTheme(theme: Theme) {
	switch (theme) {
		case Theme.SYSTEM:
			setModeCurrent(getModeOsPrefers());
			break;
		case Theme.DARK:
			setModeUserPrefers(false);
			setModeCurrent(false);
			break;
		case Theme.LIGHT:
			setModeUserPrefers(true);
			setModeCurrent(true);
			break;
	}
}

// This function is used to generate a random string. It is used to generate popup ids.
const randomStringLength: number = 10;
const generatedRandomStrings: string[] = [];

export function generateRandomString() {
	const characters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
	const charactersLength = characters.length;
	let result = '';

	for (let i = 0; i < randomStringLength; i++) {
		result += characters.charAt(Math.floor(Math.random() * charactersLength));
	}

	if (generatedRandomStrings.includes(result)) {
		return generateRandomString();
	}
	generatedRandomStrings.push(result);

	return result;
}

// This function is used to bridge the gap between the frontend and tauri. It is used to call functions from the backend and handle errors.
export async function bridge<R extends BaseResult>(func: string, args: { [key: string]: unknown }): Promise<R | undefined> {
	try {
		const result: R = await invoke(
			func,
			args
		);

		toast.set({
			type: InfoType.SUCCESS,
			message: result.message,
			timeout: 5000
		} as Toast);

		return result;
	} catch (e) {
		toast.set({
			type: InfoType.ERROR,
			message: e as string,
			timeout: 5000
		} as Toast);
	}
}
