import {CSSProperties} from 'react';
import {AbsoluteFill, Img, OffthreadVideo, interpolate, staticFile, useCurrentFrame} from 'remotion';

import {COLORS, CodeLanguage, codeColors, codeSamples} from '../data';
import {mono} from '../styles';
import {Atmosphere, CodeWindow, DesignCanvas, EditorialPanel, Fade, TinyLabel, enter} from './Primitives';
import {Presenter} from './Presenter';

const canvas: CSSProperties = {position: 'absolute', inset: 0, overflow: 'hidden'};

const SceneFrame = ({children}: {children: React.ReactNode}) => (
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

const Title = ({children, from = 0, accent = COLORS.text}: {children: React.ReactNode; from?: number; accent?: string}) => (
  <Fade from={from} duration={20} style={{position: 'absolute', left: 118, top: 114, maxWidth: 1040}}>
    <div style={{fontSize: 74, lineHeight: 1.04, letterSpacing: '-0.065em', fontWeight: 760, color: accent}}>{children}</div>
  </Fade>
);

export const HookScene = ({presenterSrc}: {presenterSrc?: string}) => {
  const frame = useCurrentFrame();
  const wordIn = enter(frame, 22, 18);
  return (
    <SceneFrame>
      <div style={{position: 'absolute', left: 118, top: 164, width: 980}}>
        <Fade from={0} duration={18}><TinyLabel color={COLORS.amber}>OpenAI Build Week</TinyLabel></Fade>
        <div style={{marginTop: 34, fontSize: 84, lineHeight: 0.97, letterSpacing: '-0.075em', fontWeight: 780, color: COLORS.text}}>
          What if code<br />could explain what<br /><span style={{color: COLORS.amber}}>it is trying to do?</span>
        </div>
        <div style={{marginTop: 42, opacity: wordIn, fontSize: 24, lineHeight: 1.45, maxWidth: 760, color: COLORS.muted}}>
          Vinglish Zero reads structure, not surface syntax.
        </div>
      </div>
      <div style={{position: 'absolute', left: 118, bottom: 106, display: 'flex', alignItems: 'center', gap: 18, opacity: enter(frame, 92, 16)}}>
        <Img src={staticFile('vinglish-zero.svg')} style={{width: 68, height: 68}} />
        <div style={{fontSize: 25, fontWeight: 720, letterSpacing: '-0.02em', color: COLORS.text}}>Vinglish Zero</div>
      </div>
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
      <Title>Four languages.<br />One behavior.</Title>
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
    ['Rust workspace', 'IR, reasoning, diagnostics'],
    ['Adapter corpus', 'Python, Java, C, Vinglish transport'],
    ['Verification', '32 deterministic semantic patterns'],
  ];
  return (
    <SceneFrame>
      <div style={{position: 'absolute', left: 118, top: 126, width: 920}}>
        <Fade from={0}><TinyLabel color={COLORS.green}>Built with Codex + GPT-5.6</TinyLabel></Fade>
        <div style={{marginTop: 30, fontSize: 66, lineHeight: 1.03, letterSpacing: '-0.064em', fontWeight: 760, color: COLORS.text}}>
          AI accelerated the build.<br />The conclusions stay inspectable.
        </div>
      </div>
      <div style={{position: 'absolute', left: 118, bottom: 128, width: 880, display: 'grid', gap: 14}}>
        {cards.map(([label, detail], index) => {
          const progress = enter(frame, 70 + index * 28, 18);
          return (
            <EditorialPanel key={label} style={{padding: '20px 26px', opacity: progress, transform: `translateY(${(1 - progress) * 14}px)`}}>
              <div style={{fontSize: 22, fontWeight: 720, letterSpacing: '-0.025em'}}>{label}</div>
              <div style={{marginTop: 5, color: '#776c5d', fontSize: 16}}>{detail}</div>
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
      <EditorialPanel style={{position: 'absolute', left: 118, top: 484, width: 1120, padding: '28px 34px', background: '#151C23', color: COLORS.text, opacity: command, transform: `translateY(${(1 - command) * 20}px)`}}>
        <div style={{...mono, fontSize: 35, letterSpacing: '-0.04em'}}><span style={{color: COLORS.cyan}}>$</span> vz query "filter THEN reduce"</div>
      </EditorialPanel>
      <div style={{position: 'absolute', left: 154, top: 635, opacity: results}}>
        <TinyLabel color={COLORS.green}>match</TinyLabel>
        <div style={{marginTop: 16, fontSize: 36, fontWeight: 720, letterSpacing: '-0.045em', color: COLORS.text}}>filter_map_reduce</div>
        <div style={{marginTop: 11, fontSize: 22, color: COLORS.muted}}>Filter → Mapper → Reducer</div>
      </div>
    </SceneFrame>
  );
};

export const EndingScene = ({presenterSrc}: {presenterSrc?: string}) => {
  const frame = useCurrentFrame();
  const reveal = interpolate(frame, [20, 86], [0, 1], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  return (
    <SceneFrame>
      <div style={{position: 'absolute', left: 118, top: 176, width: 1080, opacity: reveal}}>
        <div style={{fontSize: 72, lineHeight: 1.03, letterSpacing: '-0.066em', fontWeight: 770, color: COLORS.text}}>
          Different syntax.<br /><span style={{color: COLORS.amber}}>Same meaning.</span>
        </div>
        <div style={{marginTop: 34, fontSize: 23, color: COLORS.muted}}>github.com/Shiviatrix/VinglishZero</div>
      </div>
      <div style={{position: 'absolute', left: 118, bottom: 132, width: 158, height: 158, opacity: reveal, borderRadius: 28, background: '#17130F', border: `2px solid ${COLORS.amber}`, display: 'grid', placeItems: 'center'}}>
        <Img src={staticFile('vinglish-zero.svg')} style={{width: 126, height: 126}} />
      </div>
      {presenterSrc ? <Presenter src={presenterSrc} startFrom={169 * 30} compact /> : null}
    </SceneFrame>
  );
};
