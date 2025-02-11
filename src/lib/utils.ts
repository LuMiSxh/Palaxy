import { type AppData, FeatureFlag, Theme } from '$types/appdata';
import { browser } from '$app/environment';

const LIGHT = 'alya';
const DARK = 'alya-dark';

export function setTheme(theme: Theme) {
	if (!browser) return;

	switch (theme) {
		case Theme.LIGHT:
			document.documentElement.classList.remove('dark');
			document.documentElement.setAttribute('data-theme', LIGHT);
			break;
		case Theme.DARK:
			document.documentElement.classList.add('dark');
			document.documentElement.setAttribute('data-theme', DARK);
			break;
		case Theme.SYSTEM:
			if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
				document.documentElement.classList.add('dark');
				document.documentElement.setAttribute('data-theme', DARK);
			} else {
				document.documentElement.classList.remove('dark');
				document.documentElement.setAttribute('data-theme', LIGHT);
			}
			break;
	}
}

export function convertToTitleCase(str: string): string {
	if (!str) {
		return '';
	}
	return str.toLowerCase().replace(/\b\w/g, (s) => s.toUpperCase());
}

export function ffIsEnabled(appData: AppData, flag: FeatureFlag): boolean {
	return appData.featureFlags.includes(flag);
}
