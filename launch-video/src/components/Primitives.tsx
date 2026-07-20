import {CSSProperties, ReactNode} from 'react';
import {interpolate, spring, useCurrentFrame, useVideoConfig} from 'remotion';

import {COLORS} from '../data';
import {mono} from '../styles';

export const clamp = (value: number, min = 0, max = 1) => Math.min(Math.max(value, min), max);

export const enter = (frame: number, at: number, duration = 18) =>
  interpolate(frame, [at, at + duration], [0, 1], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

export const exit = (frame: number, at: number, duration = 18) =>
  1 - interpolate(frame, [at, at + duration], [0, 1], {
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

export const Grain = () => (
  <svg width="100%" height="100%" style={{position: 'absolute', inset: 0, opacity: 0.12, mixBlendMode: 'multiply'}}>
    <filter id="paper-grain">
      <feTurbulence type="fractalNoise" baseFrequency="0.88" numOctaves="3" stitchTiles="stitch" />
      <feColorMatrix type="saturate" values="0" />
    </filter>
    <rect width="100%" height="100%" filter="url(#paper-grain)" fill="#D4C2A8" />
  </svg>
);

export const Atmosphere = ({accent = COLORS.amber}: {accent?: string}) => {
  const rules = Array.from({length: 21}, (_, index) => index * 96);
  return (
    <>
      <div style={{position: 'absolute', inset: 0, background: COLORS.background}} />
      <svg width="1920" height="1080" style={{position: 'absolute', inset: 0, opacity: 0.22}}>
        {rules.map((position) => <line key={`h-${position}`} x1="0" y1={position} x2="1920" y2={position} stroke={COLORS.grid} strokeWidth="1" />)}
        {rules.map((position) => <line key={`v-${position}`} x1={position} y1="0" x2={position} y2="1080" stroke={COLORS.grid} strokeWidth="1" />)}
        <rect x="72" y="64" width="1776" height="952" fill="none" stroke={COLORS.faint} strokeWidth="1" />
      </svg>
      <div style={{position: 'absolute', left: 72, top: 64, height: 5, width: 156, background: accent}} />
    </>
  );
};

export const Eyebrow = ({children, color = COLORS.muted}: {children: ReactNode; color?: string}) => (
  <div
    style={{
      fontSize: 18,
      lineHeight: 1,
      letterSpacing: '0.16em',
      color,
      fontWeight: 650,
      textTransform: 'uppercase',
    }}
  >
    {children}
  </div>
);

export const EditorialPanel = ({
  children,
  style,
  radius = 28,
}: {
  children: ReactNode;
  style?: CSSProperties;
  radius?: number;
}) => (
  <div
    style={{
      background: COLORS.surface,
      color: COLORS.ink,
      border: `2px solid ${COLORS.surfaceRaised}`,
      borderRadius: radius,
      boxShadow: `6px 6px 0 ${COLORS.grid}`,
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
  dim = 0,
}: {
  language: string;
  lines: string[];
  accent: string;
  highlight?: number[];
  scale?: number;
  dim?: number;
}) => {
  return (
    <EditorialPanel style={{padding: 0, overflow: 'hidden', opacity: 1 - dim, borderColor: accent, boxShadow: `7px 7px 0 ${COLORS.ink}`}} radius={5}>
      <div
        style={{
          height: 56 * scale,
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          padding: `0 ${22 * scale}px`,
          borderBottom: `2px solid ${COLORS.surfaceRaised}`,
          background: COLORS.surfaceRaised,
        }}
      >
        {[0, 1, 2].map((index) => (
          <span key={index} style={{width: 9 * scale, height: 9 * scale, borderRadius: 99, background: index === 0 ? accent : COLORS.faint}} />
        ))}
          <span style={{marginLeft: 10 * scale, color: COLORS.ink, fontSize: 15 * scale, fontWeight: 760, letterSpacing: '0.12em'}}>{language}</span>
      </div>
      <div style={{padding: `${20 * scale}px ${24 * scale}px ${22 * scale}px`}}>
        {lines.map((line, index) => {
          const active = highlight.includes(index);
          return (
            <div
              key={`${language}-${index}`}
              style={{
                ...mono,
                height: 28 * scale,
                display: 'flex',
                alignItems: 'center',
                gap: 18 * scale,
                padding: `0 ${8 * scale}px`,
                borderRadius: 7 * scale,
                color: active ? COLORS.ink : index === 0 ? '#4E453B' : '#6F6457',
                background: active ? `${accent}24` : 'transparent',
                boxShadow: active ? `inset 2px 0 0 ${accent}` : 'none',
                whiteSpace: 'pre',
                fontSize: 17 * scale,
                lineHeight: 1,
              }}
            >
              <span style={{color: '#4a596b', width: 16 * scale, textAlign: 'right', fontSize: 12 * scale}}>{index + 1}</span>
              <span>{line || ' '}</span>
            </div>
          );
        })}
      </div>
    </EditorialPanel>
  );
};

export const TinyLabel = ({children, color = COLORS.muted}: {children: ReactNode; color?: string}) => (
  <div style={{fontSize: 14, lineHeight: 1, letterSpacing: '0.14em', fontWeight: 700, color, textTransform: 'uppercase'}}>{children}</div>
);

export const StatusDot = ({color = COLORS.green, active = true}: {color?: string; active?: boolean}) => {
  const frame = useCurrentFrame();
  const glow = active ? 0.5 + Math.sin(frame / 7) * 0.2 : 0;
  return <span style={{width: 9, height: 9, borderRadius: 99, background: active ? color : COLORS.faint, transform: `scale(${0.92 + glow * 0.08})`}} />;
};

export const Line = ({
  x1,
  y1,
  x2,
  y2,
  color = COLORS.blue,
  from = 0,
  width = 2,
  opacity = 1,
  dashed = false,
}: {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  color?: string;
  from?: number;
  width?: number;
  opacity?: number;
  dashed?: boolean;
}) => {
  const frame = useCurrentFrame();
  const progress = enter(frame, from, 16);
  const dx = x2 - x1;
  const dy = y2 - y1;
  return (
    <svg style={{position: 'absolute', inset: 0, width: '100%', height: '100%', overflow: 'visible', pointerEvents: 'none'}}>
      <line
        x1={x1}
        y1={y1}
        x2={x1 + dx * progress}
        y2={y1 + dy * progress}
        stroke={color}
        strokeWidth={width}
        strokeOpacity={opacity * progress}
        strokeDasharray={dashed ? '5 8' : undefined}
      />
    </svg>
  );
};

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
  return <div style={{opacity: progress, transform: `translateY(${(1 - progress) * 20}px)`, ...style}}>{children}</div>;
};
