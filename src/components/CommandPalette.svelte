<script lang="ts">
	import { keyboard } from '$lib/keyboard';
	import { onMount, tick } from 'svelte';
	import { IconChevronRight } from '@tabler/icons-svelte';
	import { slide } from 'svelte/transition';
	import type Command from '$types/command';
	import { keyHint } from '$states/keyhint.svelte';

	interface Props {
		commands?: Command[];
		placeholderText?: string;
		onCommandSelect?: (command: Command) => void;
		showPalette?: boolean;
	}

	let {
		commands = [],
		placeholderText = 'Search commands...',
		onCommandSelect = () => {
		},
		showPalette = $bindable(false)
	}: Props = $props();

	let value = $state('');
	let selectedIndex = $state(0);

	let inp: HTMLInputElement | null = $state(null);
	let listItemRefs: HTMLButtonElement[] = $state([]);

	let commandStack: Command[] = $state([]);

	function getAllCommands(commandList: Command[]): Command[] {
		let allCommands: Command[] = [];

		for (const command of commandList || []) {
			allCommands.push(command);
			if (!command.subcommands) continue;

			if (command.subcommands?.length > 0) {
				allCommands = [...allCommands, ...getAllCommands(command.subcommands)];
			}
		}

		return allCommands;
	}

	let filteredCommands = $derived.by(() => {
		// When searching, get results from all levels
		if (value.trim() !== '') {
			const allCommands = getAllCommands(commands);
			return allCommands.filter(item =>
				item.name.toLowerCase().includes(value.toLowerCase())
			);
		}

		// When not searching, show the current navigation level
		const currentCommands = commandStack.length > 0
			? commandStack[commandStack.length - 1].subcommands
			: commands;

		return currentCommands || [];
	});

	// When the filtered commands change, reset the selected index when it is out of bounds
	$effect(() => {
		if (selectedIndex >= filteredCommands.length) {
			selectedIndex = filteredCommands.length - 1;
		}
	});

	// Scroll to selected index
	$effect(() => {
		// This runs when selectedIndex changes
		if (selectedIndex >= 0 && listItemRefs[selectedIndex]) {
			// Set scrolling flag to true before starting scroll

			// Wait for the next tick to scroll
			tick().then(() => {
				listItemRefs[selectedIndex]?.scrollIntoView({
					behavior: 'smooth',
					block: 'nearest'
				});
			});
		}
	});

	function handleStack(): void {
		if (commandStack.length === 0) {
			keyHint.addKey("escape", "Close command palette");
		} else {
			keyHint.addKey("escape", "Go back");
		}
	}

	function executeCommand(command: Command) {
		if (command.subcommands) {
			commandStack.push(command);
			selectedIndex = 0;
			value = '';
		} else {
			command.action?.();
			onCommandSelect?.(command);
			showPalette = false;
		}
		handleStack()
	}

	function goBack() {
		if (commandStack.length > 0) {
			commandStack.pop();
			selectedIndex = 0;
			value = '';
		} else {
			showPalette = false;
		}
		handleStack()
	}

	// Register keyboard shortcuts on component mount
	onMount(() => {
		// Get the current key hints, clear them and add new ones
		const keyHints = keyHint.get();
		keyHint.clear()
		keyHint
			.addKey("arrowdown", "Navigate down")
			.addKey("arrowup", "Navigate up")
			.addKey("enter", "Select command")
			.addKey("escape", "Close command palette")
			.addKey("space", "Close command palette");

		// Focus the input
		if (inp) inp.focus();
		// Use smartRegister to handle all keyboard shortcuts
		const unregister = keyboard.smartRegister(
			[
				// Regular key handlers
				['arrowdown', (event) => {
					event.preventDefault();
					if (filteredCommands.length > 0) {
						selectedIndex = (selectedIndex + 1) % filteredCommands.length;
					}
				}],

				['arrowup', (event) => {
					event.preventDefault();
					if (filteredCommands.length > 0) {
						selectedIndex = (selectedIndex - 1 + filteredCommands.length) % filteredCommands.length;
					}
				}],

				['enter', (event) => {
					event.preventDefault();
					event.stopPropagation();
					if (filteredCommands.length === 0) return;
					executeCommand(filteredCommands[selectedIndex]);
				}],

				['escape', () => {
					goBack();
				}],

				['space', (event) => {
					event.preventDefault();
					// Reset all states
					commandStack = [];
					handleStack()
					selectedIndex = 0;
					value = '';
					showPalette = false;
				}]
			],
			[
				// Except handlers
				[
					['arrowdown', 'arrowup', 'enter', 'escape', 'tab', 'shift', 'ctrl', 'alt', 'meta'],
					() => inp?.focus(),
					'/test'
				]
			]
		);

		return () => {
			unregister();
			// Restore the key hints
			keyHint.set(keyHints);
		}
	});
</script>

<div class="fixed left-0 top-0 h-screen w-screen bg-background-dark/90 flex items-center justify-center z-30"
		 onclick={() => showPalette = false} role="button" tabindex="0" onkeydown={() => null}>
	<div class="command-palette card flex flex-col z-50">
		<input
			type="text"
			bind:value={value}
			bind:this={inp}
			placeholder={placeholderText}
			class="input"
		/>
		<div class="divider">
		</div>

		{#if commandStack.length > 0}
			<div class="breadcrumb text-sm text-gray-500 mb-2 flex gap-2">
				{#each commandStack as cmd, i}
					<button class="btn btn-xs btn-primary-soft cursor-pointer"
									onclick={(event) => {event.stopPropagation(); goBack()}}>
						{cmd.name}
					</button>
					{i < commandStack.length - 1 ? ' > ' : ''}
				{/each}
			</div>
		{/if}

		<ul class="mt-2 w-full flex-1 overflow-y-auto" role="listbox">
			{#if filteredCommands.length > 0}
				{#each filteredCommands as item, i (item.name)}
					<button
						role="option"
						aria-selected={i === selectedIndex}
						class="btn-bare w-full flex !justify-start text-left cursor-pointer {i === selectedIndex ? 'bg-primary text-white' : ''}"
						onmouseover={() => selectedIndex = i}
						onfocus={() => {}}
						onclick={(event) => {event.stopPropagation(); executeCommand(item)}}
						tabindex={i === selectedIndex ? 0 : -1}
						bind:this={listItemRefs[i]}
						transition:slide={{duration: 350, delay: i * 20}}
					>
						{#if item.icon}
							<item.icon size="22" class="mr-3 {i === selectedIndex ? 'stroke-white' : 'stroke-content-tertiary'}" />
						{/if}
						<span>
								<span class="font-medium {i === selectedIndex ? 'text-white' : 'text-content-tertiary'}">{item.name}</span>
							{#if item.description}
									<div
										class="text-sm {i === selectedIndex ? 'text-white' : 'text-content-tertiary'}">{item.description}</div>
								{/if}
							</span>
						{#if item.subcommands}
							<IconChevronRight size="20" class="ml-auto {i === selectedIndex ? 'stroke-white' : 'stroke-content-tertiary'}" />
						{/if}
					</button>
				{/each}
			{:else}
				<div class="p-2 text-center text-white">No commands found</div>
			{/if}
		</ul>
	</div>
</div>

<style>
    .command-palette {
        width: 50vw;
        padding: 12px;
        min-height: 200px;
        max-height: 90vh;
    }

    ul {
        padding: 0;
        margin: 0;
        list-style: none;
        min-height: 0;
    }
</style>
