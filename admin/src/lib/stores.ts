import { writable } from 'svelte/store';
import type { EffectsState, WifiNetwork, HudState } from './types';

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

export interface PresetPhrase {
  text: string;
  face?: string;
}

export const PRESET_PHRASES: PresetPhrase[] = [
  { text: "I AM THE I C." },
  { text: "Hello, Lunar traveler." },
  { text: "Welcome to the Mall Station!" },
  { text: "Follow me to the check-in terminal and get ready for an adventure in The Mall!" },
  { text: "Hello again, why are you still here?", face: "angry" },
  { text: "You need to sign in using this terminal!" },
  { text: "Umm.. there seems to be a problem." },
  { text: "Not to worry mister, I know another way, follow me." },
  { text: "Oh my, What a day!" },
  { text: "I love helping people who have no clue what they are doing." },
  { text: "Everyone comes here, Union Plaza is the best, Ha ha.." },
  { text: "Have you seen Jackie recently? I miss her." },
  { text: "Did you know that four legs are better than two. he he he." },
  { text: "I am happy to stay here.. yes." },
  { text: "Good by fellow Moon traveler, Good byy!" },
];
