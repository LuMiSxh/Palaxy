import { stepRegistry } from '$lib/steps/registry.svelte';
import { Step1, Step2, Step3, Step4, Step5, Step6, Step7, Step8 } from '$components/convert';
import {
	IconFolder,
	IconLineScan,
	IconSettings,
	IconFileZip,
	IconFileText,
	IconFilter,
	IconPencil,
	IconHandStop,
} from '@tabler/icons-svelte';
import convState from '$states/converter.svelte';
import { wrapper } from '$lib/utils';
import { commands } from '$types';

/**
 * Step 1: Select Source Folder
 */
stepRegistry.register({
	id: 'select-source',
	title: 'Choose Source Material',
	description: 'Select the directory containing your manga images.',
	icon: IconFolder,
	component: Step1,

	canProceed: () => {
		return convState.source !== null && convState.source !== '';
	},

	canGoBack: () => false, // Can't go back from first step

	onEnter: async () => {
		// Reset conversion state when entering first step
		await wrapper(commands.convStateReset());
		convState.reset();
	},
});

/**
 * Step 2: Analysis
 */
stepRegistry.register({
	id: 'analysis',
	title: 'Analysis',
	description: 'Analyze the source material for potential issues and improvements.',
	icon: IconLineScan,
	component: Step2,

	canProceed: () => {
		// Can proceed only if there are no blocking negatives
		return convState.analysisNegatives.length === 0;
	},
});

/**
 * Step 3: Project Setup
 */
stepRegistry.register({
	id: 'project-setup',
	title: 'Project Setup',
	description: 'Configure project name and output location.',
	icon: IconSettings,
	component: Step3,

	canProceed: () => {
		// Require name and target location
		return (
			convState.name !== null &&
			convState.name !== '' &&
			convState.target !== null &&
			convState.target !== ''
		);
	},
});

/**
 * Step 4: Bundling
 */
stepRegistry.register({
	id: 'bundling',
	title: 'Bundling',
	description: 'Choose bundling strategy and configure volume structure.',
	icon: IconFileZip,
	component: Step4,

	canProceed: () => {
		// Require at least one volume to be defined
		return convState.chapterSizes.length > 0;
	},
});

/**
 * Step 5: Output Format
 */
stepRegistry.register({
	id: 'output-format',
	title: 'Output Format',
	description: 'Configure file format, reading direction, and image processing.',
	icon: IconFileText,
	component: Step5,

	canProceed: () => {
		// Always can proceed from output format step
		return true;
	},
});

/**
 * Step 6: Filter Images
 */
stepRegistry.register({
	id: 'filter',
	title: 'Filter Images',
	description: 'Reorder images, exclude unwanted pages, and set cover images.',
	icon: IconFilter,
	component: Step6,

	canProceed: () => {
		// Always can proceed from this step (filtering is optional)
		return true;
	},
});

/**
 * Step 7: Review
 */
stepRegistry.register({
	id: 'review',
	title: 'Review',
	description: 'Review all settings before starting the conversion.',
	icon: IconPencil,
	component: Step7,

	canProceed: () => {
		// Can proceed to conversion
		return true;
	},
});

/**
 * Step 8: Conversion
 */
stepRegistry.register({
	id: 'conversion',
	title: 'Conversion',
	description: 'Converting your material. Please wait.',
	icon: IconHandStop,
	component: Step8,

	hidden: true, // Not shown in step list initially
	skipInProgress: false,

	canGoBack: () => false, // Can't go back during conversion

	canProceed: () => false, // Can't proceed during conversion
});
