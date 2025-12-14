<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { open } from '@tauri-apps/plugin-dialog';
	import converter from '$states/converter.svelte';
	import convState from '$states/converter.svelte';
	import { truncatePath, wrapper } from '$lib/utils';
	import { IconFolder, IconFolderOpen, IconCheck } from '@tabler/icons-svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';
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
			variant="glass"
			class="group relative max-h-[700px] overflow-hidden outline-none"
			onclick={!convState.source ? select : undefined}
			data-keyhint={!convState.source ? `enter;${$t`Select location`}` : undefined}
		>
			<div class="relative z-10 flex flex-col">
				<VStack gap="lg" align="center" class="text-center">
					<div
						class="from-accent-500/20 to-accent-500/5 mx-auto flex h-40 w-40 items-center justify-center rounded-full bg-linear-to-br transition-all duration-500"
						class:group-hover:scale-110={!convState.source}
					>
						{#if !convState.source}
							<IconFolder size={80} class="text-accent-500" />
						{:else}
							<IconFolderOpen size={80} class="text-accent-500" />
						{/if}
					</div>

					<VStack gap="sm" align="center" class="max-w-lg text-center">
						<h2 class="text-3xl font-bold">
							{#if !convState.source}
								{$t`Select Your Source`}
							{:else}
								{$t`Source Selected`}
							{/if}
						</h2>
						<p class="text-muted text-base">
							{#if !convState.source}
								{$t`Choose the directory containing your manga images to begin the conversion process`}
							{:else}
								{$t`Your source folder is ready for analysis`}
							{/if}
						</p>
					</VStack>

					{#if convState.source}
						<!-- Selected Path Display -->
						<div class="bg-surface-2 w-full max-w-2xl rounded-lg p-6">
							<VStack gap="md">
								<HStack gap="md" align="center">
									<div
										class="from-accent-500/30 to-accent-500/10 flex h-14 w-14 shrink-0 items-center justify-center rounded-lg bg-linear-to-br"
									>
										<IconFolderOpen size={28} class="text-accent-500" />
									</div>
									<VStack gap="sm" class="flex-1 overflow-hidden">
										<span class="text-sm font-semibold">{$t`Selected Folder`}</span>
										<div
											class="bg-surface-0 w-full truncate rounded px-4 py-3 font-mono text-sm"
											title={convState.source}
										>
											{truncatePath(convState.source, 80)}
										</div>
									</VStack>
								</HStack>

								<!-- Change Selection Button -->
								<Button variant="outline" onclick={select} tabindex={-1} class="w-full">
									<HStack gap="sm" align="center">
										<IconFolder size={18} />
										<span>{$t`Change Selection`}</span>
									</HStack>
								</Button>
							</VStack>
						</div>

						<!-- Ready Indicator -->
						<div
							class="w-full max-w-2xl rounded-lg border border-green-700/30 bg-linear-to-br from-green-700/10 to-green-700/5 p-5"
						>
							<HStack gap="md" align="center">
								<div
									class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full bg-green-700/20"
								>
									<IconCheck size={20} class="text-success" />
								</div>
								<VStack gap="xs" class="flex-1">
									<span class="text-base font-semibold">{$t`Ready to Proceed`}</span>
									<span class="text-muted text-sm">
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
