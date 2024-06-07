import { writable } from 'svelte/store';
import type { AppData, Toast } from '$lib/types';
import { appDataDefault, appDataKey } from '$lib/constants';
import { setTheme } from '$lib/functions';

// globally used stores
export const toast = writable<Toast | undefined>(undefined);

export const appData = writable<AppData>(
	JSON.parse(localStorage.getItem(appDataKey) || JSON.stringify({ ...appDataDefault }))
);
appData.subscribe((a) => {
	localStorage.setItem(appDataKey, JSON.stringify(a));

	setTheme(a.theme);
});

export const converterTab = writable<number>(0);

export const tabDisableBack = writable<boolean>(false);
export const tabDisableNext = writable<boolean>(false);
