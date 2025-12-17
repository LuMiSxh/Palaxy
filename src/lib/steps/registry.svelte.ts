import type { StepConfig } from '$types/step';

/**
 * Step Registry - Manages all wizard steps
 * Provides methods to register, retrieve, and navigate steps
 */
class StepRegistry {
	private steps: Map<string, StepConfig> = new Map();
	private order: string[] = [];

	/**
	 * Register a new step in the wizard
	 */
	register(step: StepConfig): void {
		if (this.steps.has(step.id)) {
			console.warn(`[StepRegistry] Step "${step.id}" is already registered. Overwriting.`);
		}

		this.steps.set(step.id, step);

		// Maintain insertion order
		if (!this.order.includes(step.id)) {
			this.order.push(step.id);
		}
	}

	/**
	 * Get a step by its ID
	 */
	getStep(id: string): StepConfig | undefined {
		return this.steps.get(id);
	}

	/**
	 * Get a step by its index in the order
	 */
	getStepByIndex(index: number): StepConfig | undefined {
		const id = this.order[index];
		return id ? this.steps.get(id) : undefined;
	}

	/**
	 * Get the index of a step by its ID
	 */
	getStepIndex(id: string): number {
		return this.order.indexOf(id);
	}

	/**
	 * Get all registered steps in order
	 */
	getAllSteps(): StepConfig[] {
		return this.order.map((id) => this.steps.get(id)!).filter(Boolean);
	}

	/**
	 * Get only visible steps (not hidden)
	 */
	getVisibleSteps(): StepConfig[] {
		return this.getAllSteps().filter((step) => !step.hidden);
	}

	/**
	 * Get steps that count towards progress
	 */
	getProgressSteps(): StepConfig[] {
		return this.getAllSteps().filter((step) => !step.hidden && !step.skipInProgress);
	}

	/**
	 * Get the total number of registered steps
	 */
	getTotalSteps(): number {
		return this.order.length;
	}

	/**
	 * Get the total number of visible steps
	 */
	getVisibleStepCount(): number {
		return this.getVisibleSteps().length;
	}

	/**
	 * Clear all registered steps
	 */
	clear(): void {
		this.steps.clear();
		this.order = [];
	}

	/**
	 * Check if a step exists
	 */
	hasStep(id: string): boolean {
		return this.steps.has(id);
	}
}

// Singleton instance
export const stepRegistry = new StepRegistry();
