import type { FileFormat } from './converter';
import { msg } from 'svelte-i18n-lingui';

export enum Theme {
	Light = 1,
	Dark = 2,
	System = -1
}

// Translations for the theme
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const _ = [msg`Light`, msg`Dark`, msg`System`];

export enum SupportedLanguages {
	English = 'en',
	German = 'de',
	Japanese = 'ja'
}

// Translations for the languages
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const __ = [msg`English`, msg`German`, msg`Japanese`];

export enum FeatureFlag {
	CHANGE_LANGUAGE = 11,
	SEARCH_MANGA = 12,
	BROWSE_AGENTS = 13
}

export interface AppData {
	theme: Theme;
	language: SupportedLanguages;
	featureFlags: Array<FeatureFlag>;
	autoPop: {
		enabled: boolean;
		// other properties
		converter: {
			conversionType: FileFormat | null;
			targetLocation: string | null;
			createNewFolder: boolean | null;
		};
	};
}

export const defaultAppData: AppData = {
	theme: Theme.System,
	language: SupportedLanguages.English,
	featureFlags: [],
	autoPop: {
		enabled: false,
		converter: {
			conversionType: null,
			targetLocation: null,
			createNewFolder: null
		}
	}
};
