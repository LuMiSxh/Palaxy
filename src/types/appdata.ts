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
}

// Translations for the languages
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const __ = [msg`English`, msg`German`];

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
	BROWSE_AGENTS = 13,

	/**
	 * Enables mouse support.
	 */
	MOUSE_SUPPORT = 14,

	/**
	 * Enables custom keybinding configuration.
	 */
	CUSTOM_KEYBINDS = 15
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
	 * Specifies whether key hints should be shown.
	 */
	showKeyHints: boolean;

	/**
	 * Specifies whether mouse support should be enabled.
	 */
	mouseSupport: boolean;

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
			createNewFolder: boolean;
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
	showKeyHints: true,
	mouseSupport: false,
	autoPop: {
		enabled: false,
		converter: {
			conversionType: null,
			targetLocation: null,
			createNewFolder: true
		}
	}
};
