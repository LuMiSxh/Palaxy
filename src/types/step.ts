/**
 * Configuration for a single step in the conversion wizard
 */
export interface StepConfig {
	/** Unique identifier for the step */
	id: string;

	/** Display title */
	title: string;

	/** Display description */
	description: string;

	/** Icon component to display */
	icon: any;

	/** Svelte component to render for this step */
	component: any;

	/** Called when entering this step */
	onEnter?: () => Promise<void> | void;

	/** Called when exiting this step */
	onExit?: () => Promise<void> | void;

	/** Determines if user can proceed to next step */
	canProceed?: () => boolean;

	/** Determines if user can go back to previous step */
	canGoBack?: () => boolean;

	/** If true, step won't be shown in navigation/progress */
	hidden?: boolean;

	/** If true, step won't count towards progress bar */
	skipInProgress?: boolean;
}

/**
 * Record of a step transition
 */
export interface StepTransition {
	/** Step ID transitioning from */
	from: string;

	/** Step ID transitioning to */
	to: string;

	/** When the transition occurred */
	timestamp: Date;
}

/**
 * Validation result for a step
 */
export interface StepValidationResult {
	/** Whether validation passed */
	valid: boolean;

	/** Error message if validation failed */
	message?: string;
}
