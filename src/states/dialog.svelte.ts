// src/states/dialog.svelte.ts
import type { Component } from 'svelte';

export interface Dialog {
	id: number;
	title: string;
	content: string | Component;
	contentProps?: Record<string, any>;
	onConfirm?: () => void;
	onCancel?: () => void;
	confirmText?: string;
	cancelText?: string;
	showClose?: boolean;
}

/**
 * Manages dialog/modal components in the application.
 */
class DialogState {
	/**
	 * Array of active dialogs.
	 * @type {Dialog[]}
	 */
	dialogs: Dialog[] = $state([]);

	/**
	 * Opens a new dialog.
	 * @param {Dialog} dialogOptions - The dialog configuration
	 * @returns {number} - The ID of the created dialog
	 */
	openDialog(dialogOptions: Omit<Dialog, 'id'>): number {
		const id = Math.floor(Math.random() * 1000000);
		const dialog: Dialog = { id, ...dialogOptions };

		this.dialogs = [...this.dialogs, dialog];
		return id;
	}

	/**
	 * Closes a dialog by its ID.
	 * @param {number} id - The ID of the dialog to close.
	 */
	closeDialog(id: number): void {
		this.dialogs = this.dialogs.filter((d) => d.id !== id);
	}

	/**
	 * Closes all active dialogs.
	 */
	closeAllDialogs(): void {
		this.dialogs = [];
	}
}

const dialogManager = new DialogState();
export default dialogManager;
export const openDialog = dialogManager.openDialog.bind(dialogManager);
