import { defaultAppData, type AppData } from '$types/appdata';
import { setLocale, setTheme } from '$lib/utils';
import { writable } from 'svelte/store';
import { browser } from '$app/environment';

/**
 * The key used for storing and retrieving application data.
 */
export const appDataKey = 'appData';

/**
 * Safely retrieves the app data from localStorage with error handling
 */
function getStoredAppData(): AppData {
	if (!browser) return { ...defaultAppData };

	try {
		const stored = localStorage.getItem(appDataKey);
		if (!stored) return { ...defaultAppData };

		const parsed = JSON.parse(stored);
		// Validate that we have a proper object
		if (typeof parsed !== 'object' || parsed === null) {
			return { ...defaultAppData };
		}

		// Merge with defaultAppData to ensure all fields exist
		return { ...defaultAppData, ...parsed };
	} catch (error) {
		console.error('Error reading app data from localStorage:', error);
		return { ...defaultAppData };
	}
}

/**
 * A Svelte store holding the application data.
 */
export const appData = writable<AppData>(getStoredAppData());

/**
 * Subscribes to the store changes.
 *
 * @remarks
 * Persists new values to local storage and adjusts the theme when the store updates.
 */
appData.subscribe((value) => {
	if (!browser || !value) return;

	try {
		localStorage.setItem(appDataKey, JSON.stringify(value));
		setTheme(value.theme);
		setLocale(value.language).then();
	} catch (error) {
		console.error('Error saving app data to localStorage:', error);
	}
});
