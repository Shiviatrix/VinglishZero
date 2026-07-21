import {CSSProperties} from 'react';

import {COLORS, HEIGHT, WIDTH} from './data';

export const absolute: CSSProperties = {
  position: 'absolute',
  inset: 0,
};

export const stage: CSSProperties = {
  ...absolute,
  width: WIDTH,
  height: HEIGHT,
  overflow: 'hidden',
  background: COLORS.background,
  color: COLORS.text,
  fontFamily: '"Baloo 2", system-ui, sans-serif',
};

export const mono: CSSProperties = {
  fontFamily: '"Space Mono", monospace',
};
