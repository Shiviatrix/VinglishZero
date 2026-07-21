import {AbsoluteFill, Sequence} from 'remotion';

import {AudioBed} from './components/AudioBed';
import {
  DeterminismScene,
  DiagnosticScene,
  EndingScene,
  HookScene,
  QueryScene,
  ReasoningScene,
  SemanticLiftScene,
  SyntaxScene,
  CodexScene,
} from './components/FilmScenes';
import {cueFrames, DURATION} from './data';
import {stage} from './styles';

export const LaunchFilm = ({
  showCaptions = false,
  presenterSrc,
}: {
  showCaptions?: boolean;
  presenterSrc?: string;
}) => {
  void showCaptions;
  return (
    <AbsoluteFill style={stage}>
      <style>
        {`@import url('https://fonts.googleapis.com/css2?family=Baloo+2:wght@500;600;700;800&family=Shrikhand&family=Space+Mono:wght@400;700&display=swap');`}
      </style>
      <AudioBed />
      <Sequence from={cueFrames.hook} durationInFrames={cueFrames.syntax - cueFrames.hook}><HookScene presenterSrc={presenterSrc} /></Sequence>
      <Sequence from={cueFrames.syntax} durationInFrames={cueFrames.semanticLift - cueFrames.syntax}><SyntaxScene presenterSrc={presenterSrc} /></Sequence>
      <Sequence from={cueFrames.semanticLift} durationInFrames={cueFrames.reasoning - cueFrames.semanticLift}><SemanticLiftScene presenterSrc={presenterSrc} /></Sequence>
      <Sequence from={cueFrames.reasoning} durationInFrames={cueFrames.diagnostics - cueFrames.reasoning}><ReasoningScene presenterSrc={presenterSrc} /></Sequence>
      <Sequence from={cueFrames.diagnostics} durationInFrames={cueFrames.determinism - cueFrames.diagnostics}><DiagnosticScene presenterSrc={presenterSrc} /></Sequence>
      <Sequence from={cueFrames.determinism} durationInFrames={cueFrames.codex - cueFrames.determinism}><DeterminismScene presenterSrc={presenterSrc} /></Sequence>
      <Sequence from={cueFrames.codex} durationInFrames={cueFrames.query - cueFrames.codex}><CodexScene presenterSrc={presenterSrc} /></Sequence>
      <Sequence from={cueFrames.query} durationInFrames={cueFrames.ending - cueFrames.query}><QueryScene presenterSrc={presenterSrc} /></Sequence>
      <Sequence from={cueFrames.ending} durationInFrames={DURATION - cueFrames.ending}><EndingScene presenterSrc={presenterSrc} /></Sequence>
    </AbsoluteFill>
  );
};
