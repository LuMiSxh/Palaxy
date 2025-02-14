<script lang="ts">
	import { toasts } from '$stores/toast';
	import { IconX } from '@tabler/icons-svelte';

	function getAlertType(type: "info" | "success" | "warning" | "error") {
		switch (type) {
			case "info":
				return "alert-info";
			case "success":
				return "alert-success";
			case "warning":
				return "alert-warning";
			case "error":
				return "alert-error";
			default:
				return "alert-info";
		}
	}

	function getBtnType(type: "info" | "success" | "warning" | "error") {
		switch (type) {
			case "info":
				return "btn-info";
			case "success":
				return "btn-success";
			case "warning":
				return "btn-warning";
			case "error":
				return "btn-error";
			default:
				return "btn-info";
		}
	}
</script>

<div class="toast toast-end toast-top z-50">
	{#each $toasts as toast}
		<div class="alert {getAlertType(toast.type)} flex justify-between">
			<div>
				<!--eslint-disable-next-line svelte/no-at-html-tags-->
				{@html toast.message}
			</div>
			<button class="btn btn-soft btn-xs {getBtnType(toast.type)}"
							onclick={() => toasts.update(t => t.filter(t => t.id !== toast.id))}>
				<IconX />
			</button>
		</div>
	{/each}
</div>
