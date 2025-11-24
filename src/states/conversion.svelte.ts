/**
 * Conversion state management using Svelte 5 runes
 * Tracks conversion progress and events in real-time
 */

import type {
	ConversionStartEvent,
	VolumeStartEvent,
	ImageProgressEvent,
	VolumeCompleteEvent,
	ConversionCompleteEvent,
	StatusMessageEvent,
} from '$types/bindings';

interface VolumeProgress {
	index: number;
	name: string;
	progress: number; // 0-100
	currentImage: number;
	totalImages: number;
	status: 'pending' | 'processing' | 'completed' | 'failed';
	errorMessage?: string;
}

export interface StatusMessage {
	type: 'volume_started' | 'page_added' | 'volume_finished';
	volumeIndex: number;
	volumeName: string;
	pageNumber?: number;
	totalPages?: number;
	success?: boolean;
	timestamp: number;
}

class ConversionState {
	isConverting = $state(false);
	totalVolumes = $state(0);
	currentVolumeIndex = $state<number | null>(null);
	currentVolume = $state<VolumeProgress | null>(null);
	volumes = $state<VolumeProgress[]>([]);
	completed = $state({ successful: 0, failed: 0 });
	errors = $state<Array<{ volumeName: string; error: string }>>([]);
	statusMessages = $state<StatusMessage[]>([]);
	startTime = $state<number | null>(null);
	endTime = $state<number | null>(null);
	durationSeconds = $state<number | null>(null);

	/**
	 * Handle conversion start event
	 */
	handleConversionStart(event: ConversionStartEvent) {
		this.isConverting = true;
		this.totalVolumes = event.total_volumes;
		this.startTime = Date.now();
		this.currentVolumeIndex = null;
		this.currentVolume = null;
		this.completed = { successful: 0, failed: 0 };
		this.errors = [];
		this.statusMessages = [];
		this.endTime = null;
		this.durationSeconds = null;
		this.volumes = Array.from({ length: event.total_volumes }, (_, i) => ({
			index: i,
			name: '',
			progress: 0,
			currentImage: 0,
			totalImages: 0,
			status: 'pending',
		}));
	}

	/**
	 * Handle volume start event
	 */
	handleVolumeStart(event: VolumeStartEvent) {
		const volume: VolumeProgress = {
			index: event.volume_index,
			name: event.volume_name,
			progress: 0,
			currentImage: 0,
			totalImages: 0,
			status: 'processing',
		};

		this.currentVolumeIndex = event.volume_index;
		this.currentVolume = volume;

		// Update volumes array
		if (this.volumes[event.volume_index]) {
			this.volumes[event.volume_index] = volume;
		}
	}

	/**
	 * Handle image progress event
	 */
	handleImageProgress(event: ImageProgressEvent) {
		const percentage = (event.current_image / event.total_images) * 100;

		const updatedVolume: VolumeProgress = {
			index: event.volume_index,
			name: event.volume_name,
			progress: percentage,
			currentImage: event.current_image,
			totalImages: event.total_images,
			status: 'processing',
		};

		this.currentVolume = updatedVolume;

		// Update volumes array
		if (this.volumes[event.volume_index]) {
			this.volumes[event.volume_index] = updatedVolume;
		}
	}

	/**
	 * Handle volume complete event
	 */
	handleVolumeComplete(event: VolumeCompleteEvent) {
		const volume: VolumeProgress = {
			index: event.volume_index,
			name: event.volume_name,
			progress: event.success ? 100 : this.volumes[event.volume_index]?.progress || 0,
			currentImage: this.volumes[event.volume_index]?.currentImage || 0,
			totalImages: this.volumes[event.volume_index]?.totalImages || 0,
			status: event.success ? 'completed' : 'failed',
			errorMessage: event.error_message || undefined,
		};

		// Update volumes array
		if (this.volumes[event.volume_index]) {
			this.volumes[event.volume_index] = volume;
		}

		// Update completed counts
		this.completed = {
			successful: this.completed.successful + (event.success ? 1 : 0),
			failed: this.completed.failed + (event.success ? 0 : 1),
		};

		// Add error if failed
		if (!event.success && event.error_message) {
			this.errors = [
				...this.errors,
				{
					volumeName: event.volume_name,
					error: event.error_message,
				},
			];
		}
	}

	/**
	 * Handle conversion complete event
	 */
	handleConversionComplete(event: ConversionCompleteEvent) {
		this.isConverting = false;
		this.currentVolume = null;
		this.currentVolumeIndex = null;
		this.endTime = Date.now();
		this.durationSeconds = event.duration_seconds;

		// Update final counts from event
		this.completed = {
			successful: event.successful,
			failed: event.failed,
		};
	}

	/**
	 * Handle status message event
	 */
	handleStatusMessage(event: StatusMessageEvent) {
		// StatusMessageType is a discriminated union where all fields are on the message object
		const message = event.message;
		const statusMessage: StatusMessage = {
			type: message.type,
			volumeIndex: message.volume_index,
			volumeName: message.volume_name,
			pageNumber: 'page_number' in message ? message.page_number : undefined,
			totalPages: 'total_pages' in message ? message.total_pages : undefined,
			success: 'success' in message ? message.success : undefined,
			timestamp: event.timestamp,
		};

		// Keep only last 100 messages to prevent memory issues
		if (this.statusMessages.length >= 100) {
			this.statusMessages = [...this.statusMessages.slice(-99), statusMessage];
		} else {
			this.statusMessages = [...this.statusMessages, statusMessage];
		}
	}

	/**
	 * Reset state to initial values
	 */
	reset() {
		this.isConverting = false;
		this.totalVolumes = 0;
		this.currentVolumeIndex = null;
		this.currentVolume = null;
		this.volumes = [];
		this.completed = { successful: 0, failed: 0 };
		this.errors = [];
		this.statusMessages = [];
		this.startTime = null;
		this.endTime = null;
		this.durationSeconds = null;
	}
	/**
	 * Clear only errors
	 */
	clearErrors() {
		this.errors = [];
	}
}

export const conversionState = new ConversionState();
