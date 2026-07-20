import {Audio, Sequence, staticFile} from 'remotion';

import {FPS, narrationTracks, soundCues} from '../data';

export const AudioBed = () => (
  <>
    <Audio src={staticFile('audio/ambient.wav')} volume={0.075} />
    {narrationTracks.map((track) => (
      <Sequence key={track.file} from={track.from * FPS}>
        <Audio src={staticFile(track.file)} volume={0.9} />
      </Sequence>
    ))}
    {soundCues.map((cue) => (
      <Sequence key={`${cue.from}-${cue.file}`} from={cue.from * FPS}>
        <Audio src={staticFile(cue.file)} volume={cue.volume} />
      </Sequence>
    ))}
  </>
);
