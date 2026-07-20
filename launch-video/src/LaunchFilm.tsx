import {AbsoluteFill, Sequence} from 'remotion';

import {AudioBed} from './components/AudioBed';
import {
  CompositionScene,
  DepthScene,
  DeterminismScene,
  DiagnosticScene,
  EndingScene,
  EngineScene,
  HookScene,
  PolyglotScene,
  RevealScene,
} from './components/FilmScenes';
import {cueFrames, DURATION} from './data';
import {stage} from './styles';

export const LaunchFilm = ({showCaptions = false}: {showCaptions?: boolean}) => {
  void showCaptions;
  return (
    <AbsoluteFill style={stage}>
      <AudioBed />
      <Sequence from={cueFrames.hook} durationInFrames={cueFrames.languages - cueFrames.hook}><HookScene /></Sequence>
      <Sequence from={cueFrames.languages} durationInFrames={cueFrames.reveal - cueFrames.languages}><PolyglotScene /></Sequence>
      <Sequence from={cueFrames.reveal} durationInFrames={cueFrames.engine - cueFrames.reveal}><RevealScene /></Sequence>
      <Sequence from={cueFrames.engine} durationInFrames={cueFrames.diagnostics - cueFrames.engine}><EngineScene /></Sequence>
      <Sequence from={cueFrames.diagnostics} durationInFrames={cueFrames.composition - cueFrames.diagnostics}><DiagnosticScene /></Sequence>
      <Sequence from={cueFrames.composition} durationInFrames={cueFrames.determinism - cueFrames.composition}><CompositionScene /></Sequence>
      <Sequence from={cueFrames.determinism} durationInFrames={cueFrames.depth - cueFrames.determinism}><DeterminismScene /></Sequence>
      <Sequence from={cueFrames.depth} durationInFrames={cueFrames.ending - cueFrames.depth}><DepthScene /></Sequence>
      <Sequence from={cueFrames.ending} durationInFrames={DURATION - cueFrames.ending}><EndingScene /></Sequence>
    </AbsoluteFill>
  );
};
