<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { open } from '@tauri-apps/plugin-dialog';
	import converter from '$states/converter.svelte';
	import convState from '$states/converter.svelte';
	import { truncatePath, wrapper } from '$lib/utils';
	import { IconFolder, IconFolderOpen, IconCheck } from '@tabler/icons-svelte';

	import { keyboard } from '$lib/keyboard';
	import { VStack, HStack, BentoGrid, BentoItem } from 'waku/layout';
	import { Badge } from 'waku/components';

	let unregisterKeyboard: () => void;

	onMount(async () => {
		// Reset
		await wrapper(commands.convStateReset());
		converter.reset();

		// Set Keyboard
		unregisterKeyboard = keyboard.smartRegister([['enter', select]]);

		// Auto-focus the BentoItem for keyboard-first navigation
		setTimeout(() => {
			const bentoItem = document.querySelector('.bento-item[data-keyhint]') as HTMLElement;
			bentoItem?.focus();
		}, 100);
	});

	onDestroy(() => {
		if (unregisterKeyboard) unregisterKeyboard();
	});

	async function select(evt: KeyboardEvent | MouseEvent | undefined = undefined) {
		evt?.stopPropagation();
		evt?.preventDefault();

		convState.source =
			(await open({
				directory: true,
				multiple: false,
			})) ?? '';

		if (converter.source !== null) {
			await wrapper(commands.convStateSet({ Source: convState.source ?? '' }));
		}
	}
</script>

<div class="h-full w-full p-3">
	<BentoGrid cols={2} density="comfortable" rows="auto 1fr" class="h-full">
		<!-- Status Summary Card -->
		{#if convState.source}
			<BentoItem glass padding="sm" class="max-h-24">
				<HStack gap="md" align="center" justify="between">
					<HStack gap="md" align="center">
						<div
							class="bg-success/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
						>
							<IconCheck size={20} class="text-success" />
						</div>
						<VStack gap="none">
							<span class="text-muted text-xs font-medium tracking-wide uppercase"
								>{$t`Status`}</span
							>
							<div class="text-success text-lg leading-tight font-semibold">{$t`Ready`}</div>
						</VStack>
					</HStack>
				</HStack>
			</BentoItem>

			<BentoItem glass padding="sm" class="max-h-24">
				<HStack gap="md" align="center" justify="between">
					<HStack gap="md" align="center">
						<div
							class="bg-accent-500/10 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
						>
							<IconFolderOpen size={20} class="text-accent-500" />
						</div>
						<VStack gap="none">
							<span class="text-muted text-xs font-medium tracking-wide uppercase"
								>{$t`Source`}</span
							>
							<div class="text-lg leading-tight font-semibold">{$t`Selected`}</div>
						</VStack>
					</HStack>
				</HStack>
			</BentoItem>
		{/if}

		<!-- Main Source Selection Card -->
		<BentoItem
			colspan={2}
			glass
			onclick={select}
			onkeydown={(e) => {
				if (e.key === ' ') {
					e.preventDefault();
					e.stopPropagation();
				}
			}}
			data-keyhint={`enter;${$t`Select source folder`}`}
			class={convState.source ? '' : 'row-span-2'}
		>
			<HStack gap="sm" align="center" class="text-muted mb-3">
				<IconFolder size={18} />
				<span class="text-xs font-bold tracking-wider uppercase">{$t`Source Folder`}</span>
				{#if convState.source}
					<div class="bg-success/20 ml-auto flex h-6 w-6 items-center justify-center rounded-full">
						<IconCheck size={14} class="text-success" />
					</div>
				{/if}
			</HStack>

			<VStack gap="sm" class={convState.source ? '' : 'h-full items-center justify-center'}>
				{#if convState.source}
					<div
						class="bg-surface-2 hover:bg-surface-1 group w-full cursor-pointer rounded-lg p-3 transition-colors"
					>
						<HStack gap="sm" align="center">
							<div
								class="bg-accent-500/20 flex h-10 w-10 shrink-0 items-center justify-center rounded-full"
							>
								<IconFolderOpen size={20} class="text-accent-500" />
							</div>
							<VStack gap="xs" class="flex-1 overflow-hidden">
								<span class="text-sm font-medium">{$t`Source selected`}</span>
								<span class="text-muted truncate font-mono text-xs" title={convState.source}>
									{truncatePath(convState.source, 80)}
								</span>
							</VStack>
						</HStack>
					</div>
					<p class="text-muted text-xs">
						{$t`Click to change the source folder or press Next to analyze`}
					</p>
				{:else}
					<VStack gap="lg" align="center" class="max-w-md text-center">
						<div class="bg-accent-500/10 flex h-16 w-16 items-center justify-center rounded-full">
							<IconFolder size={32} class="text-accent-500" />
						</div>
						<VStack gap="sm" align="center">
							<h3 class="text-xl font-semibold">{$t`Select Source Folder`}</h3>
							<p class="text-muted text-sm">
								{$t`Choose the directory containing your manga images to begin`}
							</p>
						</VStack>
						<div class="text-muted flex items-center gap-2 text-sm">
							<Badge variant="primary">⏎ {$t`Enter`}</Badge>
							<span>{$t`or click to browse`}</span>
						</div>
					</VStack>
				{/if}
			</VStack>
		</BentoItem>
	</BentoGrid>
</div>
