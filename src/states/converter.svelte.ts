import type { BundleFlag } from '$types';

/**
 * Manages the state for a converter, including source content,
 * selected bundle, recommended bundle, and chapter sizes.
 */
class ConverterState {
	/**
	 * Name under which the converted content will be saved.
	 * @type {string}
	 */
	name: string | null = $state(null);

	/**
	 * Original source or content to be converted.
	 * @type {string \| null}
	 */
	source: string | null = $state(null);

	/**
	 * Target path for the converted content.
	 * @type {string \| null}
	 */
	target: string | null = $state(null);

	/**
	 * The selected bundle flag.
	 * @type {BundleFlag \| null}
	 */
	bundle: BundleFlag | null = $state(null);

	/**
	 * The recommended bundle flag, defaults to MANUAL.
	 * @type {BundleFlag}
	 */
	bundleRecommendation: BundleFlag = $state('MANUAL');

	/**
	 * An array of chapter sizes for the conversion process.
	 * @type {number[]}
	 */
	chapterSizes: number[] = $state([]);

	/**
	 * The number of images that were excluded from the conversion.
	 * @type {number}
	 */
	excludedImages: number = $state(0);

	/**
	 * If the order of the chapters has changed during conversion.
	 * @type {number}
	 */
	changedOrder: boolean = $state(false);

	/**
	 * The number of new images generated during the conversion.
	 * @type {number}
	 */
	newImages: number = $state(0);

	/**
	 * Analysis results - negative issues that block proceeding
	 * @type {string[]}
	 */
	analysisNegatives: string[] = $state([]);

	/**
	 * Analysis results - warnings
	 * @type {string[]}
	 */
	analysisWarnings: string[] = $state([]);

	/**
	 * Analysis results - positive findings
	 * @type {string[]}
	 */
	analysisPositives: string[] = $state([]);

	/**
	 * Whether to flatten all chapters into a single volume.
	 * @type {boolean}
	 */
	flatten: boolean = $state(false);

	/**
	 * Resets the converter state to default values.
	 * @returns {void}
	 */
	reset(): void {
		this.name = '';
		this.source = null;
		this.target = null;
		this.bundle = null;
		this.bundleRecommendation = 'MANUAL';
		this.chapterSizes = [];
		this.excludedImages = 0;
		this.flatten = false;
		this.analysisNegatives = [];
		this.analysisWarnings = [];
		this.analysisPositives = [];
	}
}

export default new ConverterState();
