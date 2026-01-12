import { writable } from 'svelte/store';
import type { EffectsState, WifiNetwork, HudState, PresetPhrase } from './types';

export const effectsState = writable<EffectsState>({
  scanlines: true,
  scanline_opacity: 0.5,
  curvature: false,
  curvature_amount: 50,
  grid: false,
});

export const audioVolume = writable<number>(0.2);

export const wifiNetworks = writable<WifiNetwork[]>([]);
export const wifiScanning = writable<boolean>(false);
export const wifiConnecting = writable<boolean>(false);
export const selectedNetwork = writable<string | null>(null);

export const hudState = writable<HudState>({
  show_ip: false,
  ip_address: null,
});

// Preset phrases - loaded from server on app mount
export const presetPhrases = writable<PresetPhrase[]>([]);
export const phrasesLoaded = writable<boolean>(false);

// Edit mode for phrases
export const phrasesEditMode = writable<boolean>(false);

// Available faces - loaded from server on app mount
export const availableFaces = writable<string[]>(['default']);

// Re-export PresetPhrase type for convenience
export type { PresetPhrase } from './types';
