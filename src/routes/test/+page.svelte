<script lang="ts">
	import { IconSearch, IconSettings, IconTransform, IconUsers } from '@tabler/icons-svelte';
	import { goto } from '$app/navigation';
	import { keyboard } from '$lib/keyboard';
	import { onMount } from 'svelte';

	let value = $state('');
	let selectedIndex = $state(0);
	let inp: HTMLInputElement | null = $state(null);
	let selected: string | null = $state(null);
	let commandStack: Command[] = $state([]);

	interface Command {
		name: string;
		description: string;
		icon: any;
		action?: () => void;
		subcommands?: Command[];
	}

	let commands: Command[] = [
		{
			name: 'Convert',
			description:
				'Convert your manga images into a digital format that can be read on your favorite devices.',
			icon: IconTransform,
			action: () => goto('/convert')
		},
		{
			name: 'Search',
			description: 'Search for your favorite manga series and chapters from various sources.',
			icon: IconSearch,
			action: () => goto('/search')
		},
		{
			name: 'Agents',
			description: 'Manage your agents and their settings for better search results.',
			icon: IconUsers,
			action: () => goto('/agents')
		},
		{
			name: 'Settings',
			description: 'Configure the application settings and preferences.',
			icon: IconSettings,
			subcommands: [
				{
					name: 'General',
					description: 'General application settings and preferences.',
					icon: IconSettings,
					action: () => goto('/settings/general')
				},
				{
					name: 'Advanced',
					description: 'Advanced application settings and preferences.',
					icon: IconSettings,
					action: () => goto('/settings/advanced')
				}
			]
		}
	];

	let filteredCommands = $derived.by(() => {
		let currentCommands =
			commandStack.length > 0 ? commandStack[commandStack.length - 1].subcommands : commands;
		return (
			currentCommands?.filter((item) => item.name.toLowerCase().includes(value.toLowerCase())) || []
		);
	});

	// When the filtered commands change, reset the selected index when it is out of bounds
	$effect(() => {
		if (selectedIndex >= filteredCommands.length) {
			selectedIndex = filteredCommands.length - 1;
		}
	});

	// Register keyboard shortcuts on component mount
	onMount(() => {
		// Use smartRegister to handle all keyboard shortcuts
		return keyboard.smartRegister(
			[
				// Regular key handlers
				[
					'arrowdown',
					(event) => {
						event.preventDefault();
						selectedIndex = (selectedIndex + 1) % filteredCommands.length;
					},
					'/test'
				],

				[
					'arrowup',
					(event) => {
						event.preventDefault();
						selectedIndex = (selectedIndex - 1 + filteredCommands.length) % filteredCommands.length;
					},
					'/test'
				],

				[
					'enter',
					() => {
						if (filteredCommands.length === 0) return;

						const selectedCommand = filteredCommands[selectedIndex];
						if (selectedCommand.subcommands) {
							commandStack.push(selectedCommand);
							selectedIndex = 0;
						} else {
							selected = selectedCommand.name;
							selectedCommand.action?.();
						}
					},
					'/test'
				],

				[
					'escape',
					() => {
						if (commandStack.length > 0) {
							commandStack.pop();
							selectedIndex = 0;
						} else {
							selected = null;
						}
					},
					'/test'
				]
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
	});
</script>

<div class="flex h-screen flex-col items-center justify-center">
	<input type="text" bind:value bind:this={inp} />

	<ul class="mt-2 w-64 rounded border" role="listbox">
		{#each filteredCommands as item, i (item.name)}
			<button
				role="option"
				aria-selected={i === selectedIndex}
				class="w-full cursor-pointer p-2 text-left {i === selectedIndex
					? 'bg-blue-500 text-white'
					: 'hover:bg-gray-100'}"
				onmouseover={() => (selectedIndex = i)}
				onfocus={() => {}}
				onclick={() => {
					if (item.subcommands) {
						commandStack.push(item);
						selectedIndex = 0;
					} else {
						selected = item.name;
						item.action?.();
					}
				}}
				tabindex={i === selectedIndex ? 0 : -1}
			>
				{item.name}
			</button>
		{/each}
	</ul>

	{#if selected}
		<p class="mt-2">Selected: {selected}</p>
	{/if}
</div>
