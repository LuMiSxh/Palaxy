import { type AppData, defaultAppData } from '$types/appdata';
import { setLocale, setTheme } from '$lib/utils';
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

/**
 * The key used for storing and retrieving application data.
 */
export const appDataKey = 'appData';

/**
 * A Svelte store holding the application data.
 *
 * @remarks
 * Initializes the store from local storage if available, or uses a default value.
 */
export const appData = writable<AppData>(
	JSON.parse(
		browser
			? localStorage.getItem(appDataKey) || JSON.stringify(defaultAppData)
			: JSON.stringify(defaultAppData)
	)
);

/**
 * Subscribes to the store changes.
 *
 * @remarks
 * Persists new values to local storage and adjusts the theme when the store updates.
 */
appData.subscribe((value) => {
	if (typeof value === 'object') {
		if (browser) localStorage.setItem(appDataKey, JSON.stringify(value));

		setTheme(value.theme);
		setLocale(value.language).then();
	}
});
