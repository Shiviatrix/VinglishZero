import {CSSProperties} from 'react';
import {AbsoluteFill, Img, interpolate, spring, staticFile, useCurrentFrame, useVideoConfig} from 'remotion';

import {COLORS, CodeLanguage, codeColors, codeSamples} from '../data';
import {absolute, mono} from '../styles';
import {Atmosphere, CodeWindow, EditorialPanel, Eyebrow, Fade, Grain, Line, StatusDot, TinyLabel, enter} from './Primitives';

const canvas: CSSProperties = {position: 'absolute', inset: 0, overflow: 'hidden'};

const SceneFrame = ({children, accent = COLORS.amber}: {children: React.ReactNode; accent?: string}) => (
  <AbsoluteFill style={canvas}>
    <Atmosphere accent={accent} />
    {children}
    <Grain />
  </AbsoluteFill>
);

const activeLine = (frame: number, slot: number) => {
  const phase = Math.floor(frame / 44) % 3;
  return phase === slot;
};

export const HookScene = () => {
  const frame = useCurrentFrame();
  const {fps} = useVideoConfig();
  const codeEntrance = spring({frame: Math.max(0, frame - 168), fps, config: {damping: 20, stiffness: 90}});
  const focus = interpolate(frame, [168, 7 * fps], [0.94, 1.32], {extrapolateLeft: 'clamp', extrapolateRight: 'clamp'});
  const titleIn = enter(frame, 18, 24);
  const titleOut = 1 - enter(frame, 152, 30);
  const logoReveal = enter(frame, 32, 38);
  const logoHandoff = enter(frame, 142, 46);
  const logoExit = 1 - enter(frame, 190, 24);
  const markStroke = interpolate(logoReveal, [0, 1], [1160, 0]);
  const highlight = activeLine(frame, 0) ? [3] : activeLine(frame, 1) ? [4, 5] : [6];

  return (
    <SceneFrame accent={COLORS.amber}>
      <div style={{position: 'absolute', left: 194, top: 222, width: 314, height: 314, opacity: logoReveal * logoExit, transform: `translate(${logoHandoff * 162}px, ${-logoHandoff * 62}px) scale(${(0.82 + logoReveal * 0.18) * (1 - logoHandoff * 0.84)})`, transformOrigin: '50% 50%'}}>
        <svg width="314" height="314" viewBox="0 0 400 400" style={{position: 'absolute', inset: 0}}>
          <circle cx="200" cy="200" r="190" fill="none" stroke={COLORS.amber} strokeWidth="2" strokeDasharray="1160" strokeDashoffset={markStroke} />
          <path d="M 148,92 252,92 292,130 292,270 252,308 148,308 108,270 108,130 Z" fill="none" stroke={COLORS.surface} strokeWidth="3" strokeDasharray="728" strokeDashoffset={interpolate(logoReveal, [0, 1], [728, 0])} />
        </svg>
        <Img src={staticFile('vinglish-zero.svg')} style={{position: 'absolute', inset: 0, width: '100%', height: '100%', opacity: enter(frame, 72, 28), clipPath: `inset(${interpolate(logoReveal, [0, 1], [50, 0])}% 0 ${interpolate(logoReveal, [0, 1], [50, 0])}% 0)`}} />
      </div>
      <div style={{position: 'absolute', left: 584, top: 278, opacity: titleIn * titleOut, transform: `translateY(${(1 - titleIn) * 20}px)`}}>
        <div style={{fontSize: 66, fontWeight: 760, letterSpacing: '-0.055em', color: COLORS.text}}>Vinglish Zero</div>
        <div style={{marginTop: 24, height: 3, width: 84, background: COLORS.amber}} />
        <div style={{marginTop: 24, fontSize: 20, fontWeight: 720, letterSpacing: '0.14em', color: COLORS.muted}}>DETERMINISTIC SEMANTIC REASONING FOR SOURCE CODE</div>
      </div>
      <div style={{position: 'absolute', left: 335, top: 145, width: 1250, transform: `scale(${focus})`, transformOrigin: '48% 47%', opacity: codeEntrance}}>
        <CodeWindow language="PYTHON" lines={codeSamples.PYTHON} accent={COLORS.blue} highlight={highlight} scale={1.45} />
      </div>
      <div style={{position: 'absolute', top: 704, left: 780, display: 'flex', alignItems: 'center', gap: 14, opacity: enter(frame, 202, 18)}}>
        <StatusDot color={COLORS.cyan} />
        <TinyLabel color={COLORS.cyan}>shape detected</TinyLabel>
      </div>
    </SceneFrame>
  );
};

export const PolyglotScene = () => {
  const frame = useCurrentFrame();
  const languages: CodeLanguage[] = ['PYTHON', 'JAVA', 'C', 'VINGLISH'];
  const positions = [
    {left: 116, top: 132},
    {left: 1010, top: 132},
    {left: 116, top: 563},
    {left: 1010, top: 563},
  ];
  const highlight = frame < 210 ? [3] : frame < 390 ? [4, 5] : [6];
  return (
    <SceneFrame accent={COLORS.violet}>
      <div style={{position: 'absolute', top: 64, left: 116}}>
        <Fade from={0}><Eyebrow>four frontends</Eyebrow></Fade>
      </div>
      {languages.map((language, index) => {
        const inProgress = enter(frame, index * 18, 22);
        const position = positions[index];
        return (
          <div
            key={language}
            style={{
              position: 'absolute',
              ...position,
              width: 795,
              height: 385,
              opacity: inProgress,
              transform: `translateY(${(1 - inProgress) * 34}px) scale(${0.96 + inProgress * 0.04})`,
            }}
          >
            <CodeWindow language={language} lines={codeSamples[language]} accent={codeColors[language]} highlight={highlight} scale={0.72} />
          </div>
        );
      })}
      <div style={{position: 'absolute', bottom: 59, left: 116, right: 116, height: 2, background: COLORS.surfaceRaised, opacity: enter(frame, 190, 20)}} />
    </SceneFrame>
  );
};

const PipelineChip = ({label, color, progress, x}: {label: string; color: string; progress: number; x: number}) => (
  <div
    style={{
      position: 'absolute',
      left: x,
      top: 460,
      width: 265,
      height: 106,
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      borderRadius: 4,
      opacity: progress,
      transform: `translateY(${(1 - progress) * 38}px) scale(${0.88 + progress * 0.12})`,
      color: COLORS.ink,
      border: `2px solid ${color}`,
      background: COLORS.surface,
      boxShadow: `5px 5px 0 ${color}`,
      fontSize: 32,
      fontWeight: 690,
      letterSpacing: '-0.02em',
    }}
  >
    {label}
  </div>
);

export const RevealScene = () => {
  const frame = useCurrentFrame();
  const cards = ['PYTHON', 'JAVA', 'C', 'VINGLISH'] as CodeLanguage[];
  const positions = [120, 510, 900, 1290];
  const merge = enter(frame, 150, 44);
  const stage = Math.floor(frame / 100);
  return (
    <SceneFrame accent={COLORS.cyan}>
      <div style={{position: 'absolute', top: 76, left: 116, opacity: enter(frame, 0, 16)}}><Eyebrow color={COLORS.cyan}>one semantic representation</Eyebrow></div>
      {cards.map((card, index) => {
        const initial = enter(frame, index * 12, 18);
        const x = interpolate(merge, [0, 1], [positions[index], 782]);
        const y = interpolate(merge, [0, 1], [230 + (index % 2) * 190, 244]);
        const scale = interpolate(merge, [0, 1], [1, 0.52]);
        return (
          <div key={card} style={{position: 'absolute', left: x, top: y, width: 250, opacity: initial * (1 - merge * 0.88), transform: `scale(${scale})`, transformOrigin: 'top left'}}>
            <CodeWindow language={card} lines={codeSamples[card].slice(0, 5)} accent={codeColors[card]} highlight={[]} scale={0.44} />
          </div>
        );
      })}
      <div style={{position: 'absolute', left: 605, top: 253, width: 710, height: 182, opacity: merge, transform: `scale(${0.92 + merge * 0.08})`}}>
        <EditorialPanel style={{height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 22, borderColor: COLORS.cyan, boxShadow: `7px 7px 0 ${COLORS.cyan}`}} radius={4}>
          <div style={{width: 13, height: 13, borderRadius: 99, background: COLORS.cyan}} />
          <div style={{fontSize: 38, letterSpacing: '-0.03em', fontWeight: 720}}>semantic graph</div>
        </EditorialPanel>
      </div>
      <PipelineChip label="Filter" color={COLORS.cyan} progress={enter(frame, 265, 20)} x={420} />
      <PipelineChip label="Map" color={COLORS.violet} progress={enter(frame, 315, 20)} x={827} />
      <PipelineChip label="Reduce" color={COLORS.green} progress={enter(frame, 365, 20)} x={1234} />
      {stage >= 3 && <Line x1={685} y1={513} x2={825} y2={513} color={COLORS.cyan} from={300} width={3} />}
      {stage >= 4 && <Line x1={1092} y1={513} x2={1232} y2={513} color={COLORS.violet} from={350} width={3} />}
      <div style={{position: 'absolute', left: 658, top: 651, opacity: enter(frame, 398, 18)}}><TinyLabel color={COLORS.muted}>ordered semantic pipeline</TinyLabel></div>
    </SceneFrame>
  );
};

const TraceNode = ({
  label,
  detail,
  color,
  x,
  y,
  from,
  state = 'active',
}: {
  label: string;
  detail: string;
  color: string;
  x: number;
  y: number;
  from: number;
  state?: 'active' | 'rejected' | 'neutral';
}) => {
  const frame = useCurrentFrame();
  const progress = enter(frame, from, 16);
  const opacity = state === 'rejected' ? 0.42 * progress : progress;
  return (
    <div style={{position: 'absolute', left: x, top: y, width: 276, opacity, transform: `translateY(${(1 - progress) * 20}px) scale(${0.92 + progress * 0.08})`}}>
      <EditorialPanel style={{padding: '20px 22px', borderColor: state === 'rejected' ? COLORS.red : color, background: state === 'rejected' ? '#E6CAC0' : COLORS.surface, boxShadow: `5px 5px 0 ${state === 'rejected' ? COLORS.red : color}`}} radius={4}>
        <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between'}}>
          <TinyLabel color={state === 'rejected' ? COLORS.red : color}>{label}</TinyLabel>
          <StatusDot color={state === 'rejected' ? COLORS.red : color} active={state !== 'neutral'} />
        </div>
        <div style={{marginTop: 12, fontSize: 19, fontWeight: 590, letterSpacing: '-0.01em', color: COLORS.ink}}>{detail}</div>
      </EditorialPanel>
    </div>
  );
};

export const EngineScene = () => {
  const frame = useCurrentFrame();
  const pipelineOpacity = enter(frame, 960, 18);
  return (
    <SceneFrame accent={COLORS.violet}>
      <div style={{position: 'absolute', left: 116, top: 72}}><Fade from={0}><Eyebrow color={COLORS.violet}>the reasoning path</Eyebrow></Fade></div>
      <TraceNode label="Source" detail="filter_map_reduce" color={COLORS.blue} x={116} y={194} from={14} />
      <TraceNode label="Semantic IR" detail="loop · branch · calls" color={COLORS.cyan} x={448} y={194} from={120} />
      <TraceNode label="Facts" detail="iteration · predicate" color={COLORS.violet} x={780} y={194} from={230} />
      <TraceNode label="Evidence" detail="filter + transform" color={COLORS.amber} x={1112} y={194} from={340} />
      <Line x1={392} y1={260} x2={446} y2={260} color={COLORS.blue} from={90} width={3} />
      <Line x1={724} y1={260} x2={778} y2={260} color={COLORS.cyan} from={200} width={3} />
      <Line x1={1056} y1={260} x2={1110} y2={260} color={COLORS.violet} from={310} width={3} />
      <TraceNode label="Active" detail="filter" color={COLORS.cyan} x={430} y={510} from={490} />
      <TraceNode label="Active" detail="mapper" color={COLORS.violet} x={762} y={510} from={580} />
      <TraceNode label="Active" detail="reducer" color={COLORS.green} x={1094} y={510} from={670} />
      <TraceNode label="Rejected" detail="binary search" color={COLORS.red} x={1426} y={510} from={760} state="rejected" />
      <Line x1={1249} y1={350} x2={568} y2={508} color={COLORS.amber} from={450} opacity={0.72} dashed />
      <Line x1={1249} y1={350} x2={900} y2={508} color={COLORS.amber} from={540} opacity={0.72} dashed />
      <Line x1={1249} y1={350} x2={1232} y2={508} color={COLORS.amber} from={630} opacity={0.72} dashed />
      <Line x1={1249} y1={350} x2={1564} y2={508} color={COLORS.red} from={720} opacity={0.48} dashed />
      <div style={{position: 'absolute', left: 347, top: 830, width: 1226, height: 122, opacity: pipelineOpacity, transform: `translateY(${(1 - pipelineOpacity) * 18}px)`}}>
        <EditorialPanel style={{height: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 24, borderColor: COLORS.green, boxShadow: `8px 8px 0 ${COLORS.green}`}} radius={4}>
          <TinyLabel color={COLORS.muted}>intent</TinyLabel>
          <div style={{fontSize: 35, fontWeight: 700, letterSpacing: '-0.03em'}}>filter</div>
          <div style={{color: COLORS.faint, fontSize: 28}}>→</div>
          <div style={{fontSize: 35, fontWeight: 700, letterSpacing: '-0.03em'}}>mapper</div>
          <div style={{color: COLORS.faint, fontSize: 28}}>→</div>
          <div style={{fontSize: 35, fontWeight: 700, letterSpacing: '-0.03em'}}>reducer</div>
        </EditorialPanel>
      </div>
    </SceneFrame>
  );
};

export const DiagnosticScene = () => {
  const frame = useCurrentFrame();
  const reveal = enter(frame, 70, 20);
  return (
    <SceneFrame accent={COLORS.red}>
      <div style={{position: 'absolute', left: 116, top: 72}}><Fade from={0}><Eyebrow color={COLORS.red}>diagnostic interpretation</Eyebrow></Fade></div>
      <div style={{position: 'absolute', left: 116, top: 194, width: 680, opacity: enter(frame, 10, 18)}}>
        <CodeWindow language="PYTHON" lines={[
          'def calculate(values):',
          '    result = 0',
          '    for value in values:',
          '        result += value',
          '    return result',
          '',
          'calculate([1, "two", 3])',
        ]} accent={COLORS.red} highlight={[3, 6]} scale={1} />
      </div>
      <div style={{position: 'absolute', left: 874, top: 224, width: 904, opacity: reveal, transform: `translateX(${(1 - reveal) * 46}px)`}}>
        <EditorialPanel style={{padding: 34, borderColor: COLORS.red, boxShadow: `8px 8px 0 ${COLORS.red}`}} radius={4}>
          <div style={{display: 'flex', alignItems: 'center', gap: 13}}><StatusDot color={COLORS.red} /><TinyLabel color={COLORS.red}>TypeMismatch</TinyLabel></div>
          <div style={{...mono, marginTop: 27, color: COLORS.ink, fontSize: 25}}>cannot add integer and string</div>
          <div style={{height: 2, margin: '27px 0', background: COLORS.red}} />
          <TinyLabel color={COLORS.amber}>semantic conflict</TinyLabel>
          <div style={{marginTop: 13, fontSize: 29, lineHeight: 1.32, letterSpacing: '-0.02em'}}>The accumulator contract expects a numeric value on every loop iteration.</div>
          <div style={{display: 'flex', gap: 12, marginTop: 32}}>
            {['accumulation', 'numeric return', 'loop mutation'].map((item) => (
              <span key={item} style={{padding: '10px 15px', borderRadius: 3, background: COLORS.surfaceRaised, border: `1px solid ${COLORS.amber}`, color: COLORS.ink, fontSize: 15, fontWeight: 650}}>{item}</span>
            ))}
          </div>
        </EditorialPanel>
      </div>
      <div style={{position: 'absolute', left: 874, top: 754, width: 904, opacity: enter(frame, 250, 18)}}>
        <TinyLabel color={COLORS.muted}>evidence turns an error into a role-aware explanation</TinyLabel>
      </div>
    </SceneFrame>
  );
};

export const CompositionScene = () => {
  const frame = useCurrentFrame();
  const nodes = [
    {label: 'Filter', detail: 'value > 0', color: COLORS.cyan, x: 266, from: 20},
    {label: 'Mapper', detail: 'transform(value)', color: COLORS.violet, x: 738, from: 108},
    {label: 'Reducer', detail: 'combine(result, …)', color: COLORS.green, x: 1210, from: 196},
  ];
  return (
    <SceneFrame accent={COLORS.green}>
      <div style={{position: 'absolute', left: 116, top: 72}}><Fade from={0}><Eyebrow color={COLORS.green}>composition, preserved</Eyebrow></Fade></div>
      <div style={{position: 'absolute', left: 116, top: 194, width: 1688, height: 596}}>
        {nodes.map((node) => {
          const progress = enter(frame, node.from, 22);
          return (
            <div key={node.label} style={{position: 'absolute', left: node.x, top: 190, width: 392, opacity: progress, transform: `translateY(${(1 - progress) * 32}px)`}}>
              <EditorialPanel style={{padding: 32, height: 174, borderColor: node.color, boxShadow: `7px 7px 0 ${node.color}`}} radius={4}>
                <TinyLabel color={node.color}>{node.label}</TinyLabel>
                <div style={{...mono, color: COLORS.ink, marginTop: 25, fontSize: 25}}>{node.detail}</div>
              </EditorialPanel>
            </div>
          );
        })}
        <Line x1={658} y1={277} x2={736} y2={277} color={COLORS.cyan} from={80} width={3} />
        <Line x1={1130} y1={277} x2={1208} y2={277} color={COLORS.violet} from={168} width={3} />
      </div>
      <div style={{position: 'absolute', left: 364, top: 674, width: 1192, opacity: enter(frame, 285, 18)}}>
        <EditorialPanel style={{padding: 28, display: 'flex', justifyContent: 'center', alignItems: 'center', gap: 22, boxShadow: `7px 7px 0 ${COLORS.grid}`}} radius={4}>
          <TinyLabel color={COLORS.muted}>semantic pipeline</TinyLabel>
          <div style={{fontSize: 30, fontWeight: 700, color: COLORS.cyan}}>filter</div><span style={{color: COLORS.faint}}>→</span>
          <div style={{fontSize: 30, fontWeight: 700, color: COLORS.violet}}>mapper</div><span style={{color: COLORS.faint}}>→</span>
          <div style={{fontSize: 30, fontWeight: 700, color: COLORS.green}}>reducer</div>
        </EditorialPanel>
      </div>
    </SceneFrame>
  );
};

const ReportCard = ({x, from, label}: {x: number; from: number; label: string}) => {
  const frame = useCurrentFrame();
  const progress = enter(frame, from, 18);
  return (
    <div style={{position: 'absolute', left: x, top: 280, width: 586, opacity: progress, transform: `translateY(${(1 - progress) * 30}px)`}}>
      <EditorialPanel style={{padding: 32, borderColor: COLORS.cyan, boxShadow: `7px 7px 0 ${COLORS.cyan}`}} radius={4}>
        <div style={{display: 'flex', alignItems: 'center', justifyContent: 'space-between'}}><TinyLabel color={COLORS.muted}>{label}</TinyLabel><StatusDot color={COLORS.green} /></div>
        <div style={{...mono, marginTop: 28, color: COLORS.ink, fontSize: 25, lineHeight: 1.72}}>
          <div>{'{'}"hypothesis_id": "accumulator"{'}'}</div>
          <div>{'{'}"interpretation": "accumulator_type_conflict"{'}'}</div>
          <div>{'{'}"confidence": 85{'}'}</div>
        </div>
        <div style={{marginTop: 30, paddingTop: 22, borderTop: `2px solid ${COLORS.surfaceRaised}`, ...mono, color: COLORS.cyan, fontSize: 17}}>sha256: f81f…2b97</div>
      </EditorialPanel>
    </div>
  );
};

export const DeterminismScene = () => {
  const frame = useCurrentFrame();
  return (
    <SceneFrame accent={COLORS.cyan}>
      <div style={{position: 'absolute', left: 116, top: 72}}><Fade from={0}><Eyebrow color={COLORS.cyan}>deterministic execution</Eyebrow></Fade></div>
      <ReportCard x={286} from={36} label="analysis run 01" />
      <ReportCard x={1048} from={128} label="analysis run 02" />
      <div style={{position: 'absolute', top: 555, left: 859, opacity: enter(frame, 235, 16), color: COLORS.green, fontSize: 47, fontWeight: 740, letterSpacing: '-0.04em'}}>identical</div>
      <div style={{position: 'absolute', top: 660, left: 650, width: 620, height: 2, background: COLORS.cyan, opacity: enter(frame, 255, 16)}} />
    </SceneFrame>
  );
};

export const DepthScene = () => {
  const frame = useCurrentFrame();
  const rows = [
    {label: 'adapters', detail: 'python · java · c · vinglish transport', color: COLORS.blue},
    {label: 'semantic graph', detail: 'typed IR · facts · evidence', color: COLORS.cyan},
    {label: 'verification', detail: 'fixtures · deterministic reports', color: COLORS.violet},
    {label: 'execution', detail: 'cache · benchmarks · reproducible runs', color: COLORS.green},
  ];
  return (
    <SceneFrame accent={COLORS.violet}>
      <div style={{position: 'absolute', left: 390, top: 120, width: 1140}}>
        {rows.map((row, index) => {
          const progress = enter(frame, index * 55, 20);
          return (
            <div key={row.label} style={{height: 145, opacity: progress, transform: `translateX(${(1 - progress) * 30}px)`}}>
              <EditorialPanel style={{height: 112, padding: '0 32px', display: 'flex', alignItems: 'center', gap: 28, borderColor: row.color, boxShadow: `6px 6px 0 ${row.color}`}} radius={4}>
                <div style={{width: 12, height: 12, borderRadius: 99, background: row.color}} />
                <div style={{width: 220}}><TinyLabel color={row.color}>{row.label}</TinyLabel></div>
                <div style={{...mono, color: COLORS.muted, fontSize: 20}}>{row.detail}</div>
              </EditorialPanel>
            </div>
          );
        })}
      </div>
    </SceneFrame>
  );
};

const LogoReveal = ({frame}: {frame: number}) => {
  const construction = enter(frame, 122, 42);
  const reveal = enter(frame, 186, 38);
  const scale = interpolate(reveal, [0, 1], [0.84, 1]);
  const clipTop = interpolate(reveal, [0, 1], [50, 0]);
  const clipBottom = interpolate(reveal, [0, 1], [50, 0]);
  const strokeOffset = interpolate(construction, [0, 1], [1160, 0]);
  return (
    <div style={{position: 'absolute', left: 760, top: 204, width: 400, height: 400, opacity: Math.max(construction, reveal)}}>
      <svg width="400" height="400" viewBox="0 0 400 400" style={{position: 'absolute', inset: 0}}>
        <circle cx="200" cy="200" r="190" fill="none" stroke={COLORS.amber} strokeWidth="2" strokeDasharray="1160" strokeDashoffset={strokeOffset} />
        <path d="M 148,92 252,92 292,130 292,270 252,308 148,308 108,270 108,130 Z" fill="none" stroke={COLORS.surface} strokeWidth="3" strokeDasharray="728" strokeDashoffset={interpolate(construction, [0, 1], [728, 0])} />
        {[80, 126, 172, 218, 264, 310].map((x) => <circle key={x} cx={x} cy="48" r="4" fill={COLORS.surface} opacity={construction} />)}
      </svg>
      <Img
        src={staticFile('vinglish-zero.svg')}
        style={{
          position: 'absolute',
          inset: 0,
          width: '100%',
          height: '100%',
          opacity: reveal,
          transform: `scale(${scale})`,
          clipPath: `inset(${clipTop}% 0 ${clipBottom}% 0)`,
        }}
      />
    </div>
  );
};

export const EndingScene = () => {
  const frame = useCurrentFrame();
  const graphOpacity = 1 - enter(frame, 126, 34);
  const graphNodes = [
    {x: 640, y: 330, label: 'loop', color: COLORS.blue},
    {x: 940, y: 238, label: 'predicate', color: COLORS.cyan},
    {x: 1245, y: 330, label: 'transform', color: COLORS.violet},
    {x: 940, y: 560, label: 'reduce', color: COLORS.green},
  ];
  const fade = enter(frame, 285, 30);
  return (
    <SceneFrame accent={COLORS.amber}>
      <div style={{position: 'absolute', inset: 0, opacity: graphOpacity}}>
        {graphNodes.map((node, index) => {
          const progress = enter(frame, index * 18, 18);
          return (
            <div key={node.label} style={{position: 'absolute', left: node.x, top: node.y, width: 152, height: 74, borderRadius: 4, border: `2px solid ${node.color}`, background: COLORS.surface, display: 'flex', alignItems: 'center', justifyContent: 'center', color: COLORS.ink, opacity: progress, boxShadow: `5px 5px 0 ${node.color}`, fontSize: 18, fontWeight: 680}}>{node.label}</div>
          );
        })}
        <Line x1={792} y1={367} x2={938} y2={275} color={COLORS.blue} from={28} width={3} />
        <Line x1={1092} y1={275} x2={1243} y2={367} color={COLORS.cyan} from={50} width={3} />
        <Line x1={1092} y1={312} x2={938} y2={596} color={COLORS.violet} from={72} width={3} />
        <Line x1={1320} y1={404} x2={1092} y2={596} color={COLORS.green} from={94} width={3} />
      </div>
      <LogoReveal frame={frame} />
      <div style={{position: 'absolute', bottom: 102, left: 0, right: 0, textAlign: 'center', opacity: fade, transform: `translateY(${(1 - fade) * 16}px)`}}>
        <div style={{fontSize: 21, letterSpacing: '0.14em', fontWeight: 680, color: COLORS.muted}}>github.com/Shiviatrix/VinglishZero</div>
      </div>
    </SceneFrame>
  );
};
