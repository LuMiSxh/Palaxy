<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { IconChevronRight, IconSearch, IconCommand } from '@tabler/icons-svelte';
	import { slide, fade } from 'svelte/transition';
	import { t } from 'svelte-i18n-lingui';
	import { portalled, trapScroll, clickOutside, focusTrap } from 'waku/actions';
	import { Badge } from 'waku/components';
	import { keyboard } from '$lib/keyboard';
	import type Command from '$types/command';
	import { keyHint } from '$states/keyhint.svelte';

	interface Props {
		commands?: Command[];
		placeholderText?: string;
		onCommandSelect?: (command: Command) => void;
		showPalette?: boolean;
	}

	let {
		commands = $bindable([]),
		placeholderText = $t`Search commands...`,
		onCommandSelect = () => {},
		showPalette = $bindable(false),
	}: Props = $props();

	let value = $state('');
	let selectedIndex = $state(0);
	let listItemRefs: HTMLButtonElement[] = $state([]);
	let commandStack: Command[] = $state([]);
	let inputRef: HTMLInputElement | null = $state(null);

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
		if (value.trim() !== '') {
			const allCommands = getAllCommands(commands);
			return allCommands.filter((item) => {
				if (item.hidden) return false;
				return item.name.toLowerCase().includes(value.toLowerCase());
			});
		}
		const currentCommands =
			commandStack.length > 0 ? commandStack[commandStack.length - 1].subcommands : commands;
		return (currentCommands || []).filter((item) => !item.hidden);
	});

	$effect(() => {
		if (selectedIndex >= filteredCommands.length) selectedIndex = filteredCommands.length - 1;
		if (selectedIndex < 0 && filteredCommands.length > 0) selectedIndex = 0;
	});

	$effect(() => {
		if (selectedIndex >= 0 && listItemRefs[selectedIndex]) {
			tick().then(() => {
				listItemRefs[selectedIndex]?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
			});
		}
	});

	function executeCommand(command: Command) {
		if (command.subcommands) {
			commandStack.push(command);
			selectedIndex = 0;
			value = '';
			inputRef?.focus();
		} else {
			command.action?.();
			onCommandSelect?.(command);
			showPalette = false;
		}
	}

	function goBack() {
		if (commandStack.length > 0) {
			commandStack.pop();
			selectedIndex = 0;
			value = '';
		} else {
			showPalette = false;
		}
	}

	onMount(() => {
		// Get the current key hints, clear them and add new ones
		const unregisterKeyHints = keyHint.register(
			[
				['arrowdown', $t`Navigate down`],
				['arrowup', $t`Navigate up`],
				['enter', $t`Select`],
				['escape', $t`Close`],
				['space', $t`Close`],
			],
			true
		);

		// Focus the input
		if (inputRef) inputRef.focus();

		// Use smartRegister to handle all keyboard shortcuts
		const unregisterKeyboard = keyboard.smartRegister(
			[
				[
					'arrowdown',
					(e) => {
						e.preventDefault();
						selectedIndex = (selectedIndex + 1) % filteredCommands.length;
						return true;
					},
				],
				[
					'arrowup',
					(e) => {
						e.preventDefault();
						selectedIndex = (selectedIndex - 1 + filteredCommands.length) % filteredCommands.length;
						return true;
					},
				],
				[
					'enter',
					(e) => {
						e.preventDefault();
						e.stopPropagation();
						if (filteredCommands.length > 0) executeCommand(filteredCommands[selectedIndex]);
						return true;
					},
				],
				[
					'escape',
					() => {
						goBack();
						return true;
					},
				],
			],
			[
				// Except handlers
				[
					['arrowdown', 'arrowup', 'enter', 'escape', 'tab', 'shift', 'ctrl', 'alt', 'meta'],
					() => {
						inputRef?.focus();
						return true;
					},
				],
			]
		);

		return () => {
			unregisterKeyboard();
			unregisterKeyHints();
		};
	});
</script>

<div use:portalled>
	<div
		class="fixed inset-0 z-1000 bg-neutral-950/40 backdrop-blur-sm transition-all"
		transition:fade={{ duration: 250 }}
		aria-hidden="true"
	></div>

	<div
		class="pointer-events-none fixed inset-0 z-1001 flex items-start justify-center p-4 pt-[15vh]"
	>
		<div
			class="glass-heavy pointer-events-auto flex w-full max-w-xl flex-col overflow-hidden shadow-2xl"
			style="border-radius: var(--radius-xl); max-height: 60vh;"
			transition:slide={{ duration: 250, axis: 'y' }}
			use:clickOutside={() => (showPalette = false)}
			use:trapScroll
			use:focusTrap
		>
			<!-- Search Header -->
			<div class="border-waku-border/50 relative flex items-center border-b p-4">
				<IconSearch class="text-muted mr-3" size={20} />
				<input
					bind:this={inputRef}
					bind:value
					placeholder={placeholderText}
					class="placeholder:text-muted w-full border-none bg-transparent text-lg focus:outline-none"
					autocomplete="off"
				/>
				<Badge variant="neutral" class="pointer-events-none ml-2">ESC</Badge>
			</div>

			<!-- Breadcrumbs -->
			{#if commandStack.length > 0}
				<div
					class="bg-surface-2/50 border-waku-border/50 flex items-center gap-2 border-b px-4 py-2 text-sm"
				>
					<button
						class="hover:text-accent-500 transition-colors"
						onclick={() => {
							commandStack = [];
							value = '';
						}}
					>
						<IconCommand size={14} />
					</button>
					<span class="text-muted">/</span>
					{#each commandStack as cmd, i}
						<span class="text-accent-500 font-medium">{cmd.name}</span>
						{#if i < commandStack.length - 1}
							<span class="text-muted">/</span>
						{/if}
					{/each}
				</div>
			{/if}

			<!-- Results List -->
			<ul class="flex-1 overflow-y-auto scroll-smooth p-2" role="listbox">
				{#if filteredCommands.length > 0}
					{#each filteredCommands as item, i (item.name)}
						<button
							role="option"
							aria-selected={i === selectedIndex}
							class="group flex w-full items-center justify-between rounded-lg px-3 py-3 text-left transition-all duration-150
                                    {i === selectedIndex
								? 'bg-accent-500 text-white shadow-md'
								: 'hover:bg-surface-2 text-base'}"
							onclick={(e) => {
								e.stopPropagation();
								executeCommand(item);
							}}
							onmouseenter={() => (selectedIndex = i)}
							bind:this={listItemRefs[i]}
						>
							<div class="flex items-center gap-3 overflow-hidden">
								{#if item.icon}
									<item.icon
										size={20}
										class={i === selectedIndex ? 'text-white' : 'text-muted group-hover:text-base'}
									/>
								{/if}
								<div class="flex flex-col truncate">
									<span class="truncate font-medium">{item.name}</span>
									{#if item.description}
										<span
											class="truncate text-xs {i === selectedIndex
												? 'text-white/80'
												: 'text-muted'}"
										>
											{item.description}
										</span>
									{/if}
								</div>
							</div>

							{#if item.subcommands}
								<IconChevronRight
									size={16}
									class={i === selectedIndex ? 'text-white' : 'text-muted'}
								/>
							{/if}
						</button>
					{/each}
				{:else}
					<div class="text-muted flex flex-col items-center gap-2 p-8 text-center">
						<IconSearch size={32} class="opacity-20" />
						<p>No commands found</p>
					</div>
				{/if}
			</ul>
		</div>
	</div>
</div>
