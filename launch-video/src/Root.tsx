import {Composition} from 'remotion';

import {DURATION, FPS, HEIGHT, WIDTH} from './data';
import {LaunchFilm} from './LaunchFilm';

export const RemotionRoot = () => {
  return (
    <Composition
      id="VinglishZeroLaunch"
      component={LaunchFilm}
      durationInFrames={DURATION}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
      defaultProps={{showCaptions: false}}
    />
  );
};
