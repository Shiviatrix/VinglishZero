import {CSSProperties, ReactNode} from 'react';
import {interpolate, spring, useCurrentFrame, useVideoConfig} from 'remotion';

import {COLORS, DESIGN_HEIGHT, DESIGN_WIDTH} from '../data';
import {mono} from '../styles';

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

export const Atmosphere = () => <div style={{position: 'absolute', inset: 0, background: COLORS.background}} />;

export const EditorialPanel = ({
  children,
  style,
}: {
  children: ReactNode;
  style?: CSSProperties;
}) => (
  <div
    style={{
      background: COLORS.surface,
      color: COLORS.ink,
      borderRadius: 18,
      border: `1px solid ${COLORS.surfaceRaised}`,
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
  <EditorialPanel style={{padding: 0, overflow: 'hidden', border: `1px solid ${accent}66`}}>
    <div
      style={{
        height: 42 * scale,
        display: 'flex',
        alignItems: 'center',
        padding: `0 ${22 * scale}px`,
        background: accent,
        color: COLORS.ink,
        fontSize: 13 * scale,
        fontWeight: 760,
        letterSpacing: '0.16em',
      }}
    >
      {language}
    </div>
    <div style={{padding: `${17 * scale}px ${22 * scale}px ${19 * scale}px`}}>
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
              borderRadius: 5 * scale,
              color: active ? COLORS.ink : '#6F6457',
              background: active ? COLORS.surfaceRaised : 'transparent',
              whiteSpace: 'pre',
              fontSize: 16 * scale,
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

export const TinyLabel = ({children, color = COLORS.muted}: {children: ReactNode; color?: string}) => (
  <div style={{fontSize: 14, lineHeight: 1, letterSpacing: '0.16em', fontWeight: 720, color, textTransform: 'uppercase'}}>{children}</div>
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
