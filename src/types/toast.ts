export enum ToastType {
	SUCCESS = 'alert-success',
	ERROR = 'alert-error',
	WARNING = 'alert-warning',
	INFO = 'alert-info',
}

export default interface Toast {
	id: number;
	message: string;
	type: ToastType;
	timeout: number | null;
}
