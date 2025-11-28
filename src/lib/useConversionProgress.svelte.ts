/**
 * Composable for listening to conversion progress events
 * Sets up event listeners for real-time conversion progress tracking
 */

import { onMount, onDestroy } from 'svelte';
import { events } from '$types/bindings';
import { conversionState } from '$states/conversion.svelte';
import type { UnlistenFn } from '@tauri-apps/api/event';

/**
 * Hook to manage conversion progress event listeners
 * Automatically sets up and cleans up event listeners for all conversion events
 */
export function useConversionProgress() {
	let unlisteners: UnlistenFn[] = [];

	/**
	 * Set up all event listeners
	 */
	async function setupListeners() {
		try {
			// Conversion Start Event
			const unlistenStart = await events.conversionStartEvent.listen((event) => {
				console.log('[Conversion] Started:', event.payload);
				try {
					conversionState.handleConversionStart(event.payload);
				} catch (error) {
					console.error('[Conversion] Error handling start event:', error);
				}
			});
			unlisteners.push(unlistenStart);

			// Volume Start Event
			const unlistenVolumeStart = await events.volumeStartEvent.listen((event) => {
				console.log('[Conversion] Volume started:', event.payload);
				try {
					conversionState.handleVolumeStart(event.payload);
				} catch (error) {
					console.error('[Conversion] Error handling volume start:', error);
				}
			});
			unlisteners.push(unlistenVolumeStart);

			// Image Progress Event
			const unlistenImageProgress = await events.imageProgressEvent.listen((event) => {
				console.log('[Conversion] Image progress:', event.payload);
				try {
					conversionState.handleImageProgress(event.payload);
				} catch (error) {
					console.error('[Conversion] Error handling image progress:', error);
				}
			});
			unlisteners.push(unlistenImageProgress);

			// Volume Complete Event
			const unlistenVolumeComplete = await events.volumeCompleteEvent.listen((event) => {
				console.log('[Conversion] Volume complete:', event.payload);
				try {
					conversionState.handleVolumeComplete(event.payload);
				} catch (error) {
					console.error('[Conversion] Error handling volume complete:', error);
				}
			});
			unlisteners.push(unlistenVolumeComplete);

			// Conversion Complete Event
			const unlistenComplete = await events.conversionCompleteEvent.listen((event) => {
				console.log('[Conversion] Complete:', event.payload);
				try {
					conversionState.handleConversionComplete(event.payload);
				} catch (error) {
					console.error('[Conversion] Error handling complete:', error);
				}
			});
			unlisteners.push(unlistenComplete);

			// Status Message Event
			const unlistenStatusMessage = await events.statusMessageEvent.listen((event) => {
				console.log('[Conversion] Status message:', event.payload);
				try {
					conversionState.handleStatusMessage(event.payload);
				} catch (error) {
					console.error('[Conversion] Error handling status message:', error);
				}
			});
			unlisteners.push(unlistenStatusMessage);

			console.log('[Conversion] All event listeners registered');
		} catch (error) {
			console.error('[Conversion] Error setting up event listeners:', error);
		}
	}

	/**
	 * Clean up all event listeners
	 */
	function cleanup() {
		unlisteners.forEach((unlisten) => {
			try {
				unlisten();
			} catch (error) {
				console.error('[Conversion] Error unlistening:', error);
			}
		});
		unlisteners = [];
		console.log('[Conversion] Event listeners cleaned up');
	}

	onMount(() => {
		setupListeners();
	});

	onDestroy(() => {
		cleanup();
	});

	// Return the state directly without wrapping in getters
	// This avoids creating reactive dependencies that could cause loops
	return conversionState;
}
