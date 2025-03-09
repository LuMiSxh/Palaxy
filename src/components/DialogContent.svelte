<script lang="ts">
	import { t } from 'svelte-i18n-lingui';
	import dialogManager, { type Dialog } from '$states/dialog.svelte';
	import { IconX } from '@tabler/icons-svelte';
	import { fade, scale } from 'svelte/transition';
	import { keyboard } from '$lib/keyboard';
	import { onMount } from 'svelte';

	let { dialog }: { dialog: Dialog } = $props();
	let confirmButton: HTMLButtonElement | null = $state(null);
	let cancelButton: HTMLButtonElement | null = $state(null);

	function handleBackdropClick(event: MouseEvent) {
		event.stopPropagation();
		event.preventDefault();

		dialog.onCancel?.();
		dialogManager.closeDialog(dialog.id);
	}

	function handleEscapeKey(event: KeyboardEvent) {
		if (event.key === 'Escape' && dialogManager.dialogs.length > 0) {
			const currentDialog = dialogManager.dialogs[dialogManager.dialogs.length - 1];
			currentDialog.onCancel?.();
			dialogManager.closeDialog(currentDialog.id);
		}
	}

	function handleKeyNavigation(event: KeyboardEvent) {
		event.preventDefault();

		if (event.key === 'ArrowRight' && cancelButton && document.activeElement === cancelButton) {
			confirmButton?.focus();
		}

		if (event.key === 'ArrowLeft' && confirmButton && document.activeElement === confirmButton) {
			cancelButton?.focus();
		}

		// When no button is focused, focus the confirm button if presen else the cancel button
		if (!document.activeElement) {
			if (confirmButton) confirmButton.focus();
			else if (cancelButton) cancelButton.focus();
		}
	}

	function handleEnterKey(event: KeyboardEvent) {
		event.preventDefault();

		if (document.activeElement === confirmButton && dialog.onConfirm) {
			dialog.onConfirm();
			confirmButton?.blur();
			dialogManager.closeDialog(dialog.id);
		} else if (document.activeElement === cancelButton && dialog.onCancel) {
			dialog.onCancel();
			cancelButton?.blur();
			dialogManager.closeDialog(dialog.id);
		}
	}

	onMount(() => {
		// Focus the confirm button by default
		if (confirmButton) {
			confirmButton.focus();
		} else if (cancelButton) {
			cancelButton.focus();
		}

		return keyboard.smartRegister([
			['escape', handleEscapeKey],
			['enter', handleEnterKey],
			['arrowright', handleKeyNavigation],
			['arrowleft', handleKeyNavigation]
		]);
	});
</script>

<div
	class="fixed left-0 top-0 h-screen w-screen bg-background-dark/90 flex items-center justify-center z-40"
	transition:fade={{ duration: 200 }}
	onclick={handleBackdropClick}
	role="button"
	tabindex="-1"
	onkeydown={() => {}}
>
	<div
		class="bg-background dark:bg-background-dark w-full max-w-md rounded-lg shadow-lg"
		transition:scale={{ duration: 200, start: 0.95 }}
	>
		<div class="p-4 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center">
			<h2 class="text-lg font-semibold">{dialog.title}</h2>
			{#if dialog.showClose !== false}
				<button
					class="btn btn-ghost btn-sm p-1"
					onclick={() => dialogManager.closeDialog(dialog.id)}
				>
					<IconX size="20" />
				</button>
			{/if}
		</div>

		<div class="p-4">
			{#if typeof dialog.content === 'string'}
				<p>{dialog.content}</p>
			{:else}
				<dialog.content {...(dialog.contentProps || {})} />
			{/if}
		</div>

		<div class="p-4 border-t border-gray-200 dark:border-gray-700 flex justify-end gap-2">
			{#if dialog.onCancel}
				<button
					class="btn btn-outline"
					bind:this={cancelButton}
					onclick={() => {
                dialog.onCancel?.();
                dialogManager.closeDialog(dialog.id);
              }}
				>
					{dialog.cancelText || $t`Cancel`}
				</button>
			{/if}

			{#if dialog.onConfirm}
				<button
					class="btn btn-primary"
					bind:this={confirmButton}
					onclick={() => {
                dialog.onConfirm?.();
                dialogManager.closeDialog(dialog.id);
              }}
				>
					{dialog.confirmText || $t`Confirm`}
				</button>
			{/if}
		</div>
	</div>
</div>