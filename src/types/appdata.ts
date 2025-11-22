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
	German = 'de'
}

// Translations for the languages
// eslint-disable-next-line @typescript-eslint/no-unused-vars
const __ = [msg`English`, msg`German`];

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

			/**
			 * Image output format for conversion.
			 */
			imageFormat: 'None' | 'WebP' | 'AVIF' | null;
		};
	};
}

/**
 * Provides default values for the application data.
 */
export const defaultAppData: AppData = {
	theme: Theme.System,
	language: SupportedLanguages.English,
	showKeyHints: true,
	mouseSupport: false,
	autoPop: {
		enabled: false,
		converter: {
			conversionType: null,
			targetLocation: null,
			createNewFolder: true,
			imageFormat: null
		}
	}
};
