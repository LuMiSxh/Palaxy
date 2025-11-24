<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { commands } from '$types';
	import { t } from 'svelte-i18n-lingui';
	import { open } from '@tauri-apps/plugin-dialog';
	import converter from '$states/converter.svelte';
	import convState, { stepState } from '$states/converter.svelte';
	import { truncatePath, wrapper } from '$lib/utils';
	import { IconFolder } from '@tabler/icons-svelte';
	import { handleKeyHint } from '$states/keyhint.svelte';
	import { keyboard } from '$lib/keyboard';

	let btn: HTMLButtonElement | null = $state(null);
	let unregisterKeyboard: () => void;

	onMount(async () => {
		// Reset
		await wrapper(commands.convStateReset());
		converter.reset();
		stepState.reset();

		// Set Keyboard etc.
		btn?.focus();
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

	$effect(() => {
		stepState.disableNext = converter.source === null;
	});
</script>

<div class="card">
	<div class="card-header">
		<h3 class="text-lg font-bold">
			{$t`Select a folder to convert`}
		</h3>
	</div>
	<div class="card-body">
		<div class="">
			<label for="source-location" class="mb-2 block font-medium">{$t`Source Location`}</label>
			<div class="flex gap-3">
				<button
					id="source-location"
					class="btn btn-primary flex-1 justify-between overflow-hidden"
					bind:this={btn}
					use:handleKeyHint={{ keys: [['enter', $t`Select location`]] }}
					onclick={select}
				>
					<span class="flex items-center">
						<IconFolder class="mr-2" size={20} />
						{#if !convState.source}
							<span>{$t`Select input folder`}</span>
						{:else}
							<span class="overflow-hidden text-ellipsis"
								>{truncatePath(convState.source).split('/').pop()}</span
							>
						{/if}
					</span>
				</button>
			</div>
			{#if convState.source}
				<p
					class="text-content-secondary dark:text-content-dark-secondary mt-1 truncate text-xs"
					title={convState.source}
				>
					{truncatePath(convState.source, 60)}
				</p>
			{/if}
		</div>
	</div>
</div>
