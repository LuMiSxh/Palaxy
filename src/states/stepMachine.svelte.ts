import { stepRegistry } from '$lib/steps/registry.svelte';
import type { StepConfig, StepTransition } from '$types/step';

/**
 * Step Machine - Manages the state and navigation of the conversion wizard
 */
class StepMachine {
	currentStepId = $state<string>('select-source');
	history = $state<StepTransition[]>([]);
	isTransitioning = $state(false);

	/**
	 * Get the current step configuration
	 */
	get currentStep(): StepConfig | undefined {
		return stepRegistry.getStep(this.currentStepId);
	}

	/**
	 * Get the current step index (among visible steps)
	 */
	get currentIndex(): number {
		const visibleSteps = stepRegistry.getVisibleSteps();
		return visibleSteps.findIndex((step) => step.id === this.currentStepId);
	}

	/**
	 * Get the current step index (among all steps)
	 */
	get currentAbsoluteIndex(): number {
		return stepRegistry.getStepIndex(this.currentStepId);
	}

	/**
	 * Check if we can proceed to the next step
	 */
	get canGoNext(): boolean {
		if (this.isTransitioning) return false;

		const current = this.currentStep;
		if (!current) return false;

		// Check if there's a next step
		if (!this.hasNext()) return false;

		// Check validation
		const canProceed = current.canProceed?.() ?? true;
		return canProceed;
	}

	/**
	 * Check if we can go back to the previous step
	 */
	get canGoBack(): boolean {
		if (this.isTransitioning) return false;

		const current = this.currentStep;
		if (!current) return false;

		// Check if there's a previous step
		if (!this.hasPrevious()) return false;

		// Check if step allows going back
		const canGoBack = current.canGoBack?.() ?? true;
		return canGoBack;
	}

	/**
	 * Check if there's a next step
	 */
	hasNext(): boolean {
		const steps = stepRegistry.getVisibleSteps();
		const currentIdx = steps.findIndex((s) => s.id === this.currentStepId);
		return currentIdx >= 0 && currentIdx < steps.length - 1;
	}

	/**
	 * Check if there's a previous step
	 */
	hasPrevious(): boolean {
		const steps = stepRegistry.getVisibleSteps();
		const currentIdx = steps.findIndex((s) => s.id === this.currentStepId);
		return currentIdx > 0;
	}

	/**
	 * Navigate to the next step
	 */
	async next(): Promise<boolean> {
		if (!this.canGoNext) {
			console.warn('[StepMachine] Cannot proceed to next step');
			return false;
		}

		const steps = stepRegistry.getVisibleSteps();
		const currentIdx = steps.findIndex((s) => s.id === this.currentStepId);
		const nextStep = steps[currentIdx + 1];

		if (!nextStep) {
			console.error('[StepMachine] No next step found');
			return false;
		}

		return await this.transitionTo(nextStep.id);
	}

	/**
	 * Navigate to the previous step
	 */
	async back(): Promise<boolean> {
		if (!this.canGoBack) {
			console.warn('[StepMachine] Cannot go back to previous step');
			return false;
		}

		const steps = stepRegistry.getVisibleSteps();
		const currentIdx = steps.findIndex((s) => s.id === this.currentStepId);
		const prevStep = steps[currentIdx - 1];

		if (!prevStep) {
			console.error('[StepMachine] No previous step found');
			return false;
		}

		return await this.transitionTo(prevStep.id);
	}

	/**
	 * Transition to a specific step by ID
	 */
	async transitionTo(stepId: string): Promise<boolean> {
		if (this.isTransitioning) {
			console.warn('[StepMachine] Already transitioning');
			return false;
		}

		const targetStep = stepRegistry.getStep(stepId);
		if (!targetStep) {
			console.error(`[StepMachine] Step "${stepId}" not found`);
			return false;
		}

		this.isTransitioning = true;

		try {
			const oldStepId = this.currentStepId;

			// Exit current step
			if (this.currentStep?.onExit) {
				await this.currentStep.onExit();
			}

			// Record transition
			this.history.push({
				from: oldStepId,
				to: stepId,
				timestamp: new Date(),
			});

			// Update current step
			this.currentStepId = stepId;

			// Enter new step
			if (targetStep.onEnter) {
				await targetStep.onEnter();
			}

			console.log(`[StepMachine] Transitioned: ${oldStepId} → ${stepId}`);
			return true;
		} catch (error) {
			console.error('[StepMachine] Transition failed:', error);
			return false;
		} finally {
			this.isTransitioning = false;
		}
	}

	/**
	 * Reset the wizard to the first step
	 */
	reset(): void {
		const firstStep = stepRegistry.getVisibleSteps()[0];
		if (firstStep) {
			this.currentStepId = firstStep.id;
		}
		this.history = [];
		this.isTransitioning = false;
	}

	/**
	 * Get progress percentage (0-100)
	 */
	get progress(): number {
		const steps = stepRegistry.getProgressSteps();
		const currentIdx = steps.findIndex((s) => s.id === this.currentStepId);

		if (currentIdx < 0) return 0;
		return ((currentIdx + 1) / steps.length) * 100;
	}

	/**
	 * Get the number of steps completed
	 */
	get completedSteps(): number {
		const steps = stepRegistry.getVisibleSteps();
		const currentIdx = steps.findIndex((s) => s.id === this.currentStepId);
		return Math.max(0, currentIdx);
	}

	/**
	 * Get total number of visible steps
	 */
	get totalSteps(): number {
		return stepRegistry.getVisibleStepCount();
	}
}

// Singleton instance
export const stepMachine = new StepMachine();
