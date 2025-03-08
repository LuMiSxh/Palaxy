/**
 * Represents possible file formats for conversion.
 */
import { msg } from 'svelte-i18n-lingui';
import type { FileFormat } from '$types/bindings';

/**
 * Enumerates the themes available for the application UI.
 */
export enum Theme {
	/**
	 * Light-themed interface.
	 */
	Light = 1,

	/**
	 * Dark-themed interface.
	 */
	Dark = 2,

	/**
	 * Theme based on system settings.
	 */
	System = -1
}

// Translations for the theme
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const _ = [msg`Light`, msg`Dark`, msg`System`];

/**
 * Enumerates supported languages for the application.
 */
export enum SupportedLanguages {
	/**
	 * English language support.
	 */
	English = 'en',

	/**
	 * German language support.
	 */
	German = 'de',

	/**
	 * Japanese language support.
	 */
	Japanese = 'ja'
}

// Translations for the languages
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const __ = [msg`English`, msg`German`, msg`Japanese`];

/**
 * Enumerates feature flags for enabling or disabling features.
 */
export enum FeatureFlag {
	/**
	 * Allows language changes within the app.
	 */
	CHANGE_LANGUAGE = 11,

	/**
	 * Enables manga search functionality.
	 */
	SEARCH_MANGA = 12,

	/**
	 * Adds browsing agents feature.
	 */
	BROWSE_AGENTS = 13
}

/**
 * Defines the structure for storing application data.
 */
export interface AppData {
	/**
	 * Current theme setting of the application.
	 */
	theme: Theme;

	/**
	 * Current language setting of the application.
	 */
	language: SupportedLanguages;

	/**
	 * A list of activated feature flags.
	 */
	featureFlags: Array<FeatureFlag>;

	/**
	 * Configuration for autoPop features.
	 */
	autoPop: {
		/**
		 * Specifies whether the autoPop feature is enabled.
		 */
		enabled: boolean;
		/**
		 * Nested converter configuration details.
		 */
		converter: {
			/**
			 * Chosen file format for conversion.
			 */
			conversionType: FileFormat | null;

			/**
			 * Target location path for the output files.
			 */
			targetLocation: string | null;

			/**
			 * Whether a new folder should be created during conversion.
			 */
			createNewFolder: boolean | null;
		};
	};
}

/**
 * Provides default values for the application data.
 */
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
