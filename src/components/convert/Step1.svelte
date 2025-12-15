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
	import { Button, Badge } from 'waku/components';

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
	<BentoGrid cols={1} density="comfortable" class="h-full items-center">
		<!-- Main Selection Card -->
		<BentoItem
			glass
			class="group relative flex flex-col"
			onclick={select}
			data-keyhint={!convState.source ? `enter;${$t`Select location`}` : undefined}
		>
			<div class="relative z-10 flex flex-1 flex-col">
				<VStack gap="lg" align="center" class="text-center">
					<!-- Icon Section -->
					<div
						class="from-accent-500/20 to-accent-500/5 mx-auto flex h-20 w-20 shrink-0 items-center justify-center rounded-full bg-linear-to-br transition-all duration-500"
						class:group-hover:scale-110={!convState.source}
					>
						{#if !convState.source}
							<IconFolder size={32} class="text-accent-500" />
						{:else}
							<IconFolderOpen size={32} class="text-accent-500" />
						{/if}
					</div>

					<!-- Title Section -->
					<VStack gap="sm" align="center" class="max-w-lg text-center">
						<h2 class="text-2xl font-bold">
							{#if !convState.source}
								{$t`Select Your Source`}
							{:else}
								{$t`Source Selected`}
							{/if}
						</h2>
						<p class="text-muted text-sm">
							{#if !convState.source}
								{$t`Choose the directory containing your manga images to begin the conversion process`}
							{:else}
								{$t`Your source folder is ready for analysis`}
							{/if}
						</p>
					</VStack>

					<!-- Content Section - Fixed Height Container -->
					<div class="flex w-full max-w-2xl flex-col gap-4">
						{#if convState.source}
							<!-- Selected Path Display -->
							<div class="bg-surface-2 w-full rounded-lg p-5">
								<VStack gap="md">
									<HStack gap="md" align="center">
										<div
											class="from-accent-500/30 to-accent-500/10 flex h-12 w-12 shrink-0 items-center justify-center rounded-lg bg-linear-to-br"
										>
											<IconFolderOpen size={24} class="text-accent-500" />
										</div>
										<VStack gap="xs" class="flex-1 overflow-hidden">
											<span class="text-xs font-semibold tracking-wider uppercase"
												>{$t`Selected Folder`}</span
											>
											<div
												class="bg-surface-0 w-full truncate rounded px-3 py-2 font-mono text-sm"
												title={convState.source}
											>
												{truncatePath(convState.source, 80)}
											</div>
										</VStack>
									</HStack>

									<!-- Change Selection Button -->
									<Button style="outline" onclick={select} tabindex={-1} class="w-full">
										<HStack gap="sm" align="center">
											<IconFolder size={18} />
											<span>{$t`Change Selection`}</span>
										</HStack>
									</Button>
								</VStack>
							</div>

							<!-- Ready Indicator -->
							<div
								class="border-success/30 from-success/10 w-full rounded-lg border bg-linear-to-br to-green-700/5 p-4"
							>
								<HStack gap="md" align="center">
									<div
										class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-green-700/20"
									>
										<IconCheck size={20} class="text-success" />
									</div>
									<VStack gap="xs" class="flex-1">
										<span class="text-sm font-semibold">{$t`Ready to Proceed`}</span>
										<span class="text-muted text-xs">
											{$t`Click Next to analyze the source folder contents`}
										</span>
									</VStack>
								</HStack>
							</div>
						{:else}
							<!-- Instructions when no source selected -->
							<div class="text-muted flex items-center justify-center gap-2">
								<Badge variant="primary">{$t`Press Enter`}</Badge>
								<span class="text-sm">{$t`or click to browse`}</span>
							</div>
						{/if}
					</div>
				</VStack>
			</div>

			<!-- Decorative Background -->
			{#if !convState.source}
				<div
					class="from-accent-500/5 absolute inset-0 bg-linear-to-br to-transparent opacity-0 transition-opacity duration-500 group-hover:opacity-100"
				></div>
			{/if}
		</BentoItem>
	</BentoGrid>
</div>
