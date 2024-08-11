import { type Writable, writable } from 'svelte/store';

export type Transition<Type> = {
	From: Type;
	Via: Type;
	To: Type;
};

export type TransitionValue<Type, Value> = {
	Transition: Type;
	Value: Value;
};

class FSM<Type, Value> {
	public index: Type;
	public value: Value;
	private values: TransitionValue<Type, Value>[];
	private transitions: Transition<Type>[];

	constructor(
		transitions: Transition<Type>[],
		values: TransitionValue<Type, Value>[],
		index: Type = transitions[0].From,
		value: Value = values.find((transition) => transition.Transition === index)?.Value as Value
	) {
		this.transitions = transitions;
		this.values = values;
		this.index = index;
		this.value = value;
	}

	public transition(via: Type): Value {
		// Get the "to" state from the transition
		const to = this.transitions.find(
			(transition) => transition.From === this.index && transition.Via === via
		)?.To;

		// If the "to" state is not found, return the current state
		if (to === undefined) {
			console.warn(`Invalid transition from ${this.index} via ${via}`);
			return this.value;
		}

		// Update the current index and value
		this.index = to;
		this.value = this.values.find((value) => value.Transition === this.index)?.Value ?? this.value;

		// Return the new value
		return this.value;
	}
}

// Helper function to create a writable store from the FSM
export function createFSMStore<Type, Value>(
	transitions: Transition<Type>[],
	values: TransitionValue<Type, Value>[]
): Writable<Value | Type> {
	const { subscribe, update } = writable(new FSM(transitions, values));

	const customSubscribe = (run: (value: Value) => void) => {
		return subscribe((fsm) => {
			run(fsm.value);
		});
	};

	const customSet = (input: Type) => {
		update((fsm) => {
			fsm.transition(input);
			return fsm;
		});
	};

	const customUpdate = (fn: (value: Type) => Type) => {
		update((fsm) => {
			const input = fn(fsm.index);
			fsm.transition(input);
			return fsm;
		});
	};

	return {
		subscribe: customSubscribe,
		set: customSet,
		update: customUpdate
	};
}
