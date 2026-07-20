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
  fontFamily: '"Avenir Next", "Gill Sans", "Trebuchet MS", sans-serif',
};

export const mono: CSSProperties = {
  fontFamily: '"SFMono-Regular", "Cascadia Code", "Roboto Mono", Menlo, monospace',
};
