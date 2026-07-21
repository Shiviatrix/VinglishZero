import {CSSProperties, ReactNode} from 'react';
import {AbsoluteFill, Img, OffthreadVideo, interpolate, staticFile, useCurrentFrame} from 'remotion';

import {COLORS, CodeLanguage, codeColors, codeSamples} from '../data';
import {mono} from '../styles';
import {AccentChip, Atmosphere, CodeWindow, DesignCanvas, EditorialPanel, Fade, TinyLabel, enter} from './Primitives';
import {Presenter} from './Presenter';

const canvas: CSSProperties = {position: 'absolute', inset: 0, overflow: 'hidden'};

const SceneFrame = ({children}: {children: ReactNode}) => (
  <AbsoluteFill style={canvas}>
    <DesignCanvas>
      <Atmosphere />
      {children}
    </DesignCanvas>
  </AbsoluteFill>
);

const ManimClip = ({file}: {file: string}) => (
  <OffthreadVideo
    src={staticFile(`manim/${file}`)}
    style={{position: 'absolute', inset: 0, width: '100%', height: '100%', objectFit: 'cover'}}
  />
);

const Title = ({children, from = 0}: {children: ReactNode; from?: number}) => (
  <Fade from={from} duration={20} style={{position: 'absolute', left: 118, top: 114, maxWidth: 1040}}>
    <div style={{fontSize: 74, lineHeight: 1.04, letterSpacing: '-0.055em', fontWeight: 800, color: COLORS.text}}>{children}</div>
  </Fade>
);

export const HookScene = ({presenterSrc}: {presenterSrc?: string}) => {
  const frame = useCurrentFrame();
  const wordIn = enter(frame, 22, 18);
  return (
    <SceneFrame>
      <div style={{position: 'absolute', left: 118, top: 150, width: 910}}>
        <Fade from={0} duration={18}><AccentChip color={COLORS.amber}>OpenAI Build Week</AccentChip></Fade>
        <div style={{marginTop: 34, fontSize: 84, lineHeight: 0.97, letterSpacing: '-0.07em', fontWeight: 800, color: COLORS.text}}>
          Query code by<br />
          <span style={{color: COLORS.amber}}>fundamental meaning.</span>
        </div>
        <div style={{marginTop: 42, opacity: wordIn, fontSize: 25, lineHeight: 1.45, maxWidth: 720, color: COLORS.muted, fontWeight: 600}}>
          Deterministic intent inference across four language transports.
        </div>
      </div>
      <EditorialPanel style={{position: 'absolute', left: 118, bottom: 100, display: 'flex', alignItems: 'center', gap: 18, padding: '14px 20px', opacity: enter(frame, 92, 16)}}>
        <Img src={staticFile('vinglish-zero.svg')} style={{width: 58, height: 58}} />
        <div style={{display: 'flex', flexDirection: 'column'}}>
          <TinyLabel color={COLORS.blue}>Deterministic Code Reasoning</TinyLabel>
          <div style={{marginTop: 8, fontSize: 27, fontWeight: 800, letterSpacing: '-0.02em', color: COLORS.ink}}>Zero</div>
        </div>
      </EditorialPanel>
      {presenterSrc ? <Presenter src={presenterSrc} startFrom={0} /> : null}
    </SceneFrame>
  );
};

export const SyntaxScene = ({presenterSrc}: {presenterSrc?: string}) => {
  const frame = useCurrentFrame();
  const languages: CodeLanguage[] = ['PYTHON', 'JAVA', 'C', 'VINGLISH'];
  const positions = [
    {left: 118, top: 334},
    {left: 618, top: 334},
    {left: 118, top: 649},
    {left: 618, top: 649},
  ];
  return (
    <SceneFrame>
      <Title>One behavior.<br /><span style={{color: COLORS.amber}}>Written four ways.</span></Title>
      {languages.map((language, index) => {
        const progress = enter(frame, 26 + index * 18, 18);
        return (
          <div
            key={language}
            style={{
              position: 'absolute',
              ...positions[index],
              width: 430,
              opacity: progress,
              transform: `translateY(${(1 - progress) * 18}px)`,
            }}
          >
            <CodeWindow language={language} lines={codeSamples[language].slice(0, 5)} accent={codeColors[language]} highlight={[1, 2, 3]} scale={0.67} />
          </div>
        );
      })}
      {presenterSrc ? <Presenter src={presenterSrc} startFrom={15 * 30} compact /> : null}
    </SceneFrame>
  );
};

export const SemanticLiftScene = ({presenterSrc: _presenterSrc}: {presenterSrc?: string}) => <ManimClip file="SemanticLift.mp4" />;

export const ReasoningScene = ({presenterSrc: _presenterSrc}: {presenterSrc?: string}) => <ManimClip file="ReasoningProof.mp4" />;

export const DiagnosticScene = ({presenterSrc: _presenterSrc}: {presenterSrc?: string}) => <ManimClip file="DiagnosticReframe.mp4" />;

export const DeterminismScene = ({presenterSrc}: {presenterSrc?: string}) => (
  <AbsoluteFill style={canvas}>
    <ManimClip file="DeterminismProof.mp4" />
    <DesignCanvas><Presenter src={presenterSrc} startFrom={104 * 30} compact /></DesignCanvas>
  </AbsoluteFill>
);

export const CodexScene = ({presenterSrc}: {presenterSrc?: string}) => {
  const frame = useCurrentFrame();
  const cards = [
    {label: 'Rust Engine', detail: '~12.7K lines of deterministic Rust', color: COLORS.teal},
    {label: 'Language Transports', detail: 'Python · Java · C · Vinglish', color: COLORS.violet},
    {label: 'Verification', detail: '32 patterns · 80 registered tests', color: COLORS.green},
  ];
  return (
    <SceneFrame>
      <div style={{position: 'absolute', left: 118, top: 126, width: 920}}>
        <Fade from={0}><AccentChip color={COLORS.amber}>Built with Codex + GPT-5.6</AccentChip></Fade>
        <div style={{marginTop: 30, fontSize: 66, lineHeight: 1.03, letterSpacing: '-0.055em', fontWeight: 800, color: COLORS.text}}>
          AI accelerated the build.<br /><span style={{color: COLORS.red}}>Every conclusion is inspectable.</span>
        </div>
      </div>
      <div style={{position: 'absolute', left: 118, bottom: 128, width: 880, display: 'grid', gap: 16}}>
        {cards.map(({label, detail, color}, index) => {
          const progress = enter(frame, 70 + index * 28, 18);
          return (
            <EditorialPanel key={label} style={{padding: '16px 22px', opacity: progress, transform: `translateY(${(1 - progress) * 14}px)`}}>
              <AccentChip color={color} style={{fontSize: 12, padding: '7px 10px', boxShadow: 'none'}}>{label}</AccentChip>
              <div style={{marginTop: 11, color: COLORS.ink, fontSize: 19, fontWeight: 700}}>{detail}</div>
            </EditorialPanel>
          );
        })}
      </div>
      {presenterSrc ? <Presenter src={presenterSrc} startFrom={126 * 30} /> : null}
    </SceneFrame>
  );
};

export const QueryScene = ({presenterSrc: _presenterSrc}: {presenterSrc?: string}) => {
  const frame = useCurrentFrame();
  const command = enter(frame, 18, 20);
  const results = enter(frame, 176, 22);
  return (
    <SceneFrame>
      <Title>Search by meaning,<br />not filenames.</Title>
      <EditorialPanel style={{position: 'absolute', left: 118, top: 484, width: 1120, padding: '28px 34px', opacity: command, transform: `translateY(${(1 - command) * 20}px)`}}>
        <div style={{...mono, fontSize: 35, letterSpacing: '-0.04em', color: COLORS.ink}}><span style={{color: COLORS.red, fontWeight: 700}}>$</span> vz query "filter THEN reduce"</div>
      </EditorialPanel>
      <EditorialPanel style={{position: 'absolute', left: 118, top: 635, width: 780, padding: '22px 28px', opacity: results}}>
        <AccentChip color={COLORS.green} style={{fontSize: 12, padding: '7px 10px', boxShadow: 'none'}}>Match</AccentChip>
        <div style={{...mono, marginTop: 16, fontSize: 34, fontWeight: 700, letterSpacing: '-0.045em', color: COLORS.ink}}>filter_map_reduce</div>
        <div style={{marginTop: 11, fontSize: 22, color: COLORS.ink, fontWeight: 700}}>Filter → Mapper → Reducer</div>
        <TinyLabel color={COLORS.blue} style={{marginTop: 16}}>Cached Semantic Report · No Source Reparse</TinyLabel>
      </EditorialPanel>
    </SceneFrame>
  );
};

export const EndingScene = ({presenterSrc}: {presenterSrc?: string}) => {
  const frame = useCurrentFrame();
  const reveal = interpolate(frame, [20, 86], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  return (
    <SceneFrame>
      <div style={{position: 'absolute', left: 118, top: 176, width: 1080, opacity: reveal}}>
        <div style={{fontSize: 64, lineHeight: 1.03, letterSpacing: '-0.066em', fontWeight: 770, color: COLORS.text}}>
          Code shouldn't be searched by what it looks like.<br /><span style={{color: COLORS.amber}}>It should be searched by what it does.</span>
        </div>
        <div style={{marginTop: 34, fontSize: 26, color: COLORS.muted}}>Every conclusion backed by deterministic proof.</div>
        <div style={{marginTop: 18, fontSize: 20, color: COLORS.muted}}>github.com/Shiviatrix/VinglishZero</div>
      </div>
      <EditorialPanel style={{position: 'absolute', left: 118, bottom: 132, width: 172, height: 172, opacity: reveal, display: 'grid', placeItems: 'center', background: COLORS.paper}}>
        <Img src={staticFile('vinglish-zero.svg')} style={{width: 132, height: 132}} />
      </EditorialPanel>
      {presenterSrc ? <Presenter src={presenterSrc} startFrom={169 * 30} compact /> : null}
    </SceneFrame>
  );
};
