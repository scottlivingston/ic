export interface EffectsState {
  glow: boolean;
  glow_intensity: number;
  scanlines: boolean;
  scanline_opacity: number;
  flicker: boolean;
  flicker_amount: number;
  curvature: boolean;
  curvature_amount: number;
  grid: boolean;
}

export interface WifiNetwork {
  ssid: string;
  signal: number;
  security: string;
  in_use: boolean;
}

export interface HudState {
  show_ip: boolean;
  ip_address: string | null;
}

export interface WifiConnectResponse {
  success: boolean;
  ip_address?: string;
  error?: string;
}

export interface WifiForgetResponse {
  success: boolean;
  error?: string;
}
