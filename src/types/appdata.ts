import type { FileFormat } from './converter';

export enum Theme {
	LIGHT = 1,
	DARK = 2,
	SYSTEM = -1
}

export enum SupportedLanguages {
	ENGLISH = 1,
	GERMAN = 2
}

export enum FeatureFlag {
	CHANGE_LANGUAGE = 11
}

export interface AppData {
	theme: Theme;
	language: SupportedLanguages;
	featureFlags: Array<FeatureFlag>;
	autoFill: {
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
	theme: Theme.SYSTEM,
	language: SupportedLanguages.ENGLISH,
	featureFlags: [],
	autoFill: {
		enabled: false,
		converter: {
			conversionType: null,
			targetLocation: null,
			createNewFolder: null
		}
	}
};
