import { type AppData, defaultAppData } from '$types/appdata';
import { setTheme } from '$lib/utils';
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export const appDataKey = 'appData';

export const appData = writable<AppData>(
	JSON.parse(
		browser
			? localStorage.getItem(appDataKey) || JSON.stringify(defaultAppData)
			: JSON.stringify(defaultAppData)
	)
);

appData.subscribe((value) => {
	if (typeof value === 'object') {
		if (browser) localStorage.setItem(appDataKey, JSON.stringify(value));

		setTheme(value.theme);
		// FIXME: Uncomment as soon as lingui extract is fixed
		// setLocale(value.language).then();
	}
});
