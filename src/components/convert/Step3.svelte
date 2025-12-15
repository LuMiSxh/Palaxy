<script lang="ts">
	import { appData } from '$stores/appdata';
	import convState from '$states/converter.svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { IconFolder, IconFile, IconFolderPlus, IconCheck } from '@tabler/icons-svelte';
	import { open } from '@tauri-apps/plugin-dialog';
	import { onDestroy, onMount } from 'svelte';
	import { truncatePath, wrapper } from '$lib/utils';
	import { keyHint } from '$states/keyhint.svelte';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Input, Toggle } from 'waku/components';

	let name = $state(
		convState.name === ''
			? convState.source
				? (convState.source.split('/').pop() ?? '')
				: ''
			: (convState.name ?? '')
	);
	let targetLocation = $state(
		$appData.autoPop.enabled
			? ($appData.autoPop.converter.targetLocation ?? convState.target ?? '')
			: (convState.target ?? '')
	);
	let createFolder = $state(
		$appData.autoPop.enabled ? ($appData.autoPop.converter.createNewFolder ?? true) : true
	);

	let projectNamePlaceholder = $derived(($t`Enter project name` || '') as string);

	async function select() {
		targetLocation =
			(await open({
				directory: true,
				multiple: false,
			})) ?? '';
	}

	onMount(() => {
		return keyHint.register([['tab', $t`Navigate fields`]]);
	});

	// Update convState immediately when values change
	$effect(() => {
		convState.name = name;
		convState.target = targetLocation;
	});

	onDestroy(async () => {
		// Set Tauri AppState
		await wrapper(commands.convStateSet({ Name: name ?? '' }));
		await wrapper(commands.convStateSet({ CreateDirectory: createFolder }));
		await wrapper(commands.convStateSet({ Target: targetLocation }));
	});
</script>

<div class="h-full w-full p-3">
	<BentoGrid cols={2} density="comfortable" rows="auto auto">
		<!-- Project Name -->
		<BentoItem glass padding="sm">
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFile size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Project Name`}</span>
			</HStack>

			<VStack gap="sm">
				<Input
					id="project-name"
					bind:value={name}
					placeholder={projectNamePlaceholder}
					style="seamless"
					onkeydown={(e) => {
						if (e.key === ' ') {
							e.stopPropagation();
						}
					}}
				/>
				<p class="text-muted text-xs">
					{$t`This will be used as the base name for your output files`}
				</p>
			</VStack>
		</BentoItem>

		<!-- Create Folder Toggle -->
		<BentoItem
			glass
			padding="sm"
			onclick={() => (createFolder = !createFolder)}
			data-keyhint={`enter;${$t`Toggle`}`}
		>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFolderPlus size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Output Structure`}</span>
			</HStack>

			<div
				class="hover:bg-surface-2 group flex w-full cursor-pointer items-center justify-between rounded-lg p-3 text-left transition-colors"
			>
				<VStack gap="xs" class="flex-1">
					<span class="text-sm font-medium">
						{createFolder ? $t`Create project folder` : $t`Direct output`}
					</span>
					<span class="text-muted text-xs">
						{#if createFolder}
							{$t`Files will be placed in a new folder named after the project`}
						{:else}
							{$t`Files will be saved directly in the target location`}
						{/if}
					</span>
				</VStack>
				<div class="pointer-events-none ml-4">
					<Toggle bind:checked={createFolder} tabindex={-1} />
				</div>
			</div>
		</BentoItem>

		<!-- Target Location -->
		<BentoItem
			colspan={2}
			glass
			padding="sm"
			onclick={select}
			data-keyhint={`enter;${$t`Select location`}`}
			class="relative overflow-hidden"
		>
			<div class="relative z-10">
				<HStack gap="sm" align="center" class="mb-4">
					<div
						class="flex h-10 w-10 items-center justify-center rounded-full transition-colors {targetLocation
							? 'bg-accent-500/20'
							: ''}"
						class:text-accent-500={targetLocation}
						class:bg-surface-2={!targetLocation}
						class:text-muted={!targetLocation}
					>
						<IconFolder size={20} />
					</div>
					<span class="text-sm font-bold tracking-wider uppercase">{$t`Output Location`}</span>
					{#if targetLocation}
						<div
							class="bg-success/20 ml-auto flex h-6 w-6 items-center justify-center rounded-full"
						>
							<IconCheck size={14} class="text-success" />
						</div>
					{/if}
				</HStack>

				<VStack gap="md">
					<button
						id="target-location"
						class="bg-surface-2 hover:bg-surface-1 hover:border-accent-500/50 focus:border-accent-500 focus:ring-accent-500/20 group w-full rounded-lg border border-transparent p-4 text-left transition-all focus:ring-2 focus:outline-none"
						onclick={(e) => {
							e.stopPropagation();
							select();
						}}
						tabindex={-1}
					>
						<HStack gap="md" align="center">
							<div
								class="from-accent-500/30 to-accent-500/10 flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-linear-to-br transition-all group-hover:scale-105"
							>
								<IconFolder size={24} class="text-accent-500" />
							</div>
							<VStack gap="xs" class="flex-1 overflow-hidden">
								<span class="font-medium">
									{#if !targetLocation}
										{$t`Select output folder`}
									{:else}
										{$t`Output folder selected`}
									{/if}
								</span>
								{#if targetLocation}
									<span class="text-muted truncate font-mono text-sm" title={targetLocation}>
										{truncatePath(targetLocation, 100)}
									</span>
								{:else}
									<span class="text-muted text-sm">
										{$t`Click to browse for a folder`}
									</span>
								{/if}
							</VStack>
						</HStack>
					</button>
					<p class="text-muted text-xs leading-relaxed">
						{$t`Choose where the converted files will be saved`}
					</p>
				</VStack>
			</div>
			{#if targetLocation}
				<div
					class="from-accent-500/5 absolute inset-0 bg-linear-to-br to-transparent opacity-50"
				></div>
			{/if}
		</BentoItem>
	</BentoGrid>
</div>
