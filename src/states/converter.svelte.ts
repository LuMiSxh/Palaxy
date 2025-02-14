import type { BundleFlag } from '$types';

/**
 * Manages the state for a converter, including source content,
 * selected bundle, recommended bundle, and chapter sizes.
 */
class ConverterState {
	/**
	 * Original source or content to be converted.
	 * @type {string \| null}
	 */
	source: string | null = $state(null);

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
	 * Resets the converter state to default values.
	 * @returns {void}
	 */
	reset(): void {
		this.source = null;
		this.bundle = null;
		this.bundleRecommendation = 'MANUAL';
		this.chapterSizes = [];
	}
}

export default new ConverterState();

/**
 * Manages step-based navigation state, including current index
 * and navigation controls.
 */
class ConverterStepState {
	/**
	 * The current step index.
	 * @type {number}
	 */
	index: number = $state(0);

	/**
	 * Increment value for advancing the step index.
	 * @type {number}
	 */
	indexIncrement: number = $state(1);

	/**
	 * Decrement value for reversing the step index.
	 * @type {number}
	 */
	indexDecrement: number = $state(1);

	/**
	 * Whether the user can navigate to the previous step.
	 * @type {boolean}
	 */
	disablePrev: boolean = $state(true);

	/**
	 * Whether the user can navigate to the next step.
	 * @type {boolean}
	 */
	disableNext: boolean = $state(true);

	/**
	 * Resets the step state to its default values.
	 * @returns {void}
	 */
	reset(): void {
		this.index = 0;
		this.indexIncrement = 1;
		this.indexDecrement = 1;
		this.disablePrev = true;
		this.disableNext = true;
	}
}

export const stepState = new ConverterStepState();
