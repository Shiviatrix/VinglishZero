import {OffthreadVideo, staticFile} from 'remotion';

import {COLORS} from '../data';

export const Presenter = ({
  src,
  startFrom = 0,
  compact = false,
}: {
  src?: string;
  startFrom?: number;
  compact?: boolean;
}) => {
  if (!src) return null;

  const width = compact ? 332 : 560;
  const height = compact ? 416 : 700;
  const resolvedSrc = src.startsWith('http://') || src.startsWith('https://') ? src : staticFile(src);

  return (
    <div
      style={{
        position: 'absolute',
        right: compact ? 58 : 112,
        bottom: compact ? 54 : 86,
        width,
        height,
        overflow: 'hidden',
        borderRadius: compact ? 24 : 32,
        border: `2px solid ${COLORS.amber}`,
        background: '#17130F',
      }}
    >
      <OffthreadVideo
        src={resolvedSrc}
        startFrom={startFrom}
        muted
        style={{width: '100%', height: '100%', objectFit: 'cover'}}
      />
    </div>
  );
};
