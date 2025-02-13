<script lang="ts">
	import { toasts } from '$stores/toast';
</script>

<div class="toast toast-end toast-top z-50">
	{#each $toasts as toast (toast.id)}
		<div class="alert {toast.type}">
			<span>
				{toast.message}
			</span>
		</div>
	{/each}
</div>
