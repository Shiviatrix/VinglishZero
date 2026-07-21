import {CSSProperties, ReactNode} from 'react';
import {interpolate, spring, useCurrentFrame, useVideoConfig} from 'remotion';

import {accentTextColor, COLORS, DESIGN_HEIGHT, DESIGN_WIDTH} from '../data';
import {mono} from '../styles';

const inkBorder = `2px solid ${COLORS.border}`;

export const enter = (frame: number, at: number, duration = 18) =>
  interpolate(frame, [at, at + duration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

export const useSpringIn = (from: number, config?: {damping?: number; mass?: number; stiffness?: number}) => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  return spring({
    frame: Math.max(0, frame - from),
    fps,
    config: {damping: 18, stiffness: 120, mass: 0.8, ...config},
  });
};

export const DesignCanvas = ({children}: {children: ReactNode}) => {
  const {width, height} = useVideoConfig();
  const scale = Math.min(width / DESIGN_WIDTH, height / DESIGN_HEIGHT);

  return (
    <div
      style={{
        position: 'absolute',
        width: DESIGN_WIDTH,
        height: DESIGN_HEIGHT,
        transform: `scale(${scale})`,
        transformOrigin: 'top left',
      }}
    >
      {children}
    </div>
  );
};

export const Atmosphere = () => (
  <div
    style={{
      position: 'absolute',
      inset: 0,
      backgroundColor: COLORS.background,
    }}
  />
);

export const EditorialPanel = ({
  children,
  style,
}: {
  children: ReactNode;
  style?: CSSProperties;
}) => (
  <div
    style={{
      background: COLORS.paper,
      color: COLORS.ink,
      border: inkBorder,
      ...style,
    }}
  >
    {children}
  </div>
);

export const AccentChip = ({
  children,
  color = COLORS.green,
  style,
}: {
  children: ReactNode;
  color?: string;
  style?: CSSProperties;
}) => (
  <div
    style={{
      ...mono,
      display: 'inline-flex',
      alignItems: 'center',
      width: 'fit-content',
      padding: '9px 14px',
      border: `3px solid ${COLORS.border}`,
      color: accentTextColor(color),
      background: color,
      fontSize: 14,
      fontWeight: 700,
      letterSpacing: '0.08em',
      lineHeight: 1,
      textTransform: 'uppercase',
      ...style,
    }}
  >
    {children}
  </div>
);

export const CodeWindow = ({
  language,
  lines,
  accent,
  highlight = [],
  scale = 1,
}: {
  language: string;
  lines: string[];
  accent: string;
  highlight?: number[];
  scale?: number;
}) => (
  <EditorialPanel style={{padding: 0, overflow: 'hidden'}}>
    <div
      style={{
        ...mono,
        height: 42 * scale,
        display: 'flex',
        alignItems: 'center',
        padding: `0 ${22 * scale}px`,
        background: accent,
        color: accentTextColor(accent),
        fontSize: 13 * scale,
        fontWeight: 700,
        letterSpacing: '0.12em',
        borderBottom: inkBorder,
      }}
    >
      {language}
    </div>
    <div style={{padding: `${17 * scale}px ${22 * scale}px ${19 * scale}px`, background: COLORS.paper}}>
      {lines.map((line, index) => {
        const active = highlight.includes(index);
        return (
          <div
            key={`${language}-${index}`}
            style={{
              ...mono,
              minHeight: 26 * scale,
              display: 'flex',
              alignItems: 'center',
              padding: `0 ${9 * scale}px`,
              color: COLORS.ink,
              background: active ? COLORS.yellow : 'transparent',
              borderLeft: active ? `4px solid ${COLORS.red}` : '4px solid transparent',
              whiteSpace: 'pre',
              fontSize: 16 * scale,
              fontWeight: active ? 700 : 400,
              lineHeight: 1.35,
            }}
          >
            {line || ' '}
          </div>
        );
      })}
    </div>
  </EditorialPanel>
);

export const TinyLabel = ({
  children,
  color = COLORS.blue,
  style,
}: {
  children: ReactNode;
  color?: string;
  style?: CSSProperties;
}) => (
  <div
    style={{
      ...mono,
      fontSize: 14,
      lineHeight: 1,
      letterSpacing: '0.12em',
      fontWeight: 700,
      color,
      textTransform: 'uppercase',
      ...style,
    }}
  >
    {children}
  </div>
);

export const Fade = ({
  from,
  children,
  duration = 16,
  style,
}: {
  from: number;
  children: ReactNode;
  duration?: number;
  style?: CSSProperties;
}) => {
  const frame = useCurrentFrame();
  const progress = enter(frame, from, duration);
  return <div style={{opacity: progress, transform: `translateY(${(1 - progress) * 18}px)`, ...style}}>{children}</div>;
};
