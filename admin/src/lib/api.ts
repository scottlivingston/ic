import type { EffectsState, WifiNetwork, HudState, WifiConnectResponse, WifiForgetResponse, WifiStatus } from './types';

export async function sendSpeak(msg: string, face?: string): Promise<void> {
  try {
    const body: { msg: string; face?: string } = { msg };
    if (face) {
      body.face = face;
    }
    const response = await fetch('/api/speak', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });
    if (!response.ok) {
      console.error('Failed to send speak:', response.statusText);
    }
  } catch (err) {
    console.error('Error sending speak:', err);
  }
}

export async function sendEffects(effects: EffectsState): Promise<void> {
  try {
    const response = await fetch('/api/effects', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(effects),
    });
    if (!response.ok) {
      console.error('Failed to send effects:', response.statusText);
    }
  } catch (err) {
    console.error('Error sending effects:', err);
  }
}

export async function sendVolume(volume: number): Promise<void> {
  // Scale volume: 100% on slider = 20% actual (speaker is loud)
  const scaledVolume = volume * 0.2;
  try {
    const response = await fetch('/api/volume', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ volume: scaledVolume }),
    });
    if (!response.ok) {
      console.error('Failed to send volume:', response.statusText);
    }
  } catch (err) {
    console.error('Error sending volume:', err);
  }
}

export async function scanWifi(): Promise<WifiNetwork[]> {
  try {
    const response = await fetch('/api/wifi/scan');
    if (response.ok) {
      const data = await response.json();
      return data.networks || [];
    } else {
      console.error('Failed to scan WiFi:', response.statusText);
      return [];
    }
  } catch (err) {
    console.error('Error scanning WiFi:', err);
    return [];
  }
}

export async function connectWifi(ssid: string, password: string): Promise<WifiConnectResponse> {
  try {
    const response = await fetch('/api/wifi/connect', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ssid, password }),
    });
    return await response.json();
  } catch (err) {
    console.error('Error connecting to WiFi:', err);
    return { success: false, error: 'Connection error' };
  }
}

export async function forgetWifi(ssid: string): Promise<WifiForgetResponse> {
  try {
    const response = await fetch('/api/wifi/forget', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ ssid }),
    });
    return await response.json();
  } catch (err) {
    console.error('Error forgetting WiFi:', err);
    return { success: false, error: 'Request error' };
  }
}

export async function fetchWifiStatus(): Promise<WifiStatus> {
  try {
    const response = await fetch('/api/wifi/status');
    if (response.ok) {
      return await response.json();
    }
  } catch (err) {
    console.error('Error fetching WiFi status:', err);
  }
  return { connected: false, ssid: null, ip_address: null };
}

export async function fetchHudStatus(): Promise<HudState> {
  try {
    const response = await fetch('/api/hud/status');
    if (response.ok) {
      return await response.json();
    }
  } catch (err) {
    console.error('Error fetching HUD status:', err);
  }
  return { show_ip: false, ip_address: null };
}

export async function toggleIpHud(show: boolean): Promise<boolean> {
  try {
    const response = await fetch('/api/hud/ip', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ show }),
    });
    return response.ok;
  } catch (err) {
    console.error('Error toggling IP HUD:', err);
    return false;
  }
}
