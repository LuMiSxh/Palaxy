import { writable } from "svelte/store"
import { BundleFlag } from "$components/converter/types"

export const index = writable<number>(0)
export const disableBack = writable<boolean>(false)
export const disableNext = writable<boolean>(false)

export const loading = writable<boolean>(false)
export const source = writable<string | null>(null)
export const bundle = writable<BundleFlag | null>(null)
export const bundleRecommendation = writable<BundleFlag>(BundleFlag.MANUAL)
export const chapterSizes = writable<number[]>([])
