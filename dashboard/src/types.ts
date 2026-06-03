export interface DisplayScreen {
  id: string;
  name: string;
  width: number;
  height: number;
}

export interface OverlayTextSettings {
  enabled: boolean;
  content: string;
  position: 'above' | 'below';
  color: string;
  size: number;
}

export interface OverlayState {
  x: number;
  y: number;
  width: number;
  height: number;
  widthMode: 'custom' | 'auto';
  heightMode: 'custom' | 'auto';
}
