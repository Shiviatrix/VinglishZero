export const FPS = 30;
// The layout is authored on a stable design canvas and rendered at 4K.
export const DESIGN_WIDTH = 1920;
export const DESIGN_HEIGHT = 1080;
export const WIDTH = DESIGN_WIDTH * 2;
export const HEIGHT = DESIGN_HEIGHT * 2;
// Keep the rendered film safely below the Build Week three-minute limit.
export const DURATION = 179 * FPS;

// A restrained matte palette keeps the semantic proofs legible at 4K.
export const COLORS = {
  cream: '#F4F0E9',
  paper: '#E8E2D9',
  red: '#E97866',
  blue: '#8AB6C9',
  green: '#9BBF9A',
  orange: '#E89143',
  pink: '#D7A0A8',
  ink: '#171513',
  yellow: '#E4CC76',
  border: '#B9B1A7',
  teal: '#8AB6C9',
  violet: '#C4A7D7',
  amber: '#E89143',
  background: '#0E0D0C',
  surface: '#E8E2D9',
  surfaceRaised: '#D6CEC3',
  text: '#F4F0E9',
  muted: '#B9B1A7',
  faint: '#9BBF9A',
} as const;

const lightAccents: readonly string[] = [
  COLORS.cream,
  COLORS.paper,
  COLORS.red,
  COLORS.blue,
  COLORS.green,
  COLORS.orange,
  COLORS.pink,
  COLORS.yellow,
  COLORS.teal,
  COLORS.violet,
  COLORS.amber,
];

export const accentTextColor = (accent: string) => (lightAccents.includes(accent) ? COLORS.ink : COLORS.paper);

export type CodeLanguage = 'PYTHON' | 'JAVA' | 'C' | 'VINGLISH';

export const codeSamples: Record<CodeLanguage, string[]> = {
  PYTHON: [
    'def filter_map_reduce(values):',
    '    result = 0',
    '    for value in values:',
    '        if value > 0:',
    '            result = combine(',
    '                result, transform(value))',
    '    return result',
  ],
  JAVA: [
    'static int filter_map_reduce(int[] values) {',
    '    int result = 0;',
    '    for (int value : values) {',
    '        if (value > 0) {',
    '            result = combine(',
    '                result, transform(value));',
    '        }',
    '    } return result;',
    '}',
  ],
  C: [
    'int filter_map_reduce(int values[], int n) {',
    '    int result = 0;',
    '    for (int i = 0; i < n; i++) {',
    '        if (values[i] > 0) {',
    '            result = combine(',
    '                result, transform(values[i]));',
    '        }',
    '    } return result;',
    '}',
  ],
  VINGLISH: [
    'function filter_map_reduce(number count)',
    'returns number',
    'begin',
    '    let mutable total be 0',
    '    let mutable index be 0',
    '    repeat while index < count',
    '    begin',
    '        if index > 0',
    '            total += index * index',
    '        index += 1',
    '    end',
    '    return total',
    'end',
  ],
};

export const codeColors: Record<CodeLanguage, string> = {
  PYTHON: COLORS.teal,
  JAVA: COLORS.amber,
  C: COLORS.violet,
  VINGLISH: COLORS.green,
};

export const cueFrames = {
  hook: 0,
  syntax: 15 * FPS,
  semanticLift: 29 * FPS,
  reasoning: 52 * FPS,
  diagnostics: 78 * FPS,
  determinism: 104 * FPS,
  codex: 126 * FPS,
  query: 151 * FPS,
  ending: 169 * FPS,
} as const;

export const narrationTracks = [
  {from: 0, file: 'audio/narration-01.wav'},
  {from: 15, file: 'audio/narration-02.wav'},
  {from: 29, file: 'audio/narration-03.wav'},
  {from: 52, file: 'audio/narration-04.wav'},
  {from: 78, file: 'audio/narration-05.wav'},
  {from: 104, file: 'audio/narration-06.wav'},
  {from: 126, file: 'audio/narration-07.wav'},
  {from: 151, file: 'audio/narration-08.wav'},
  {from: 169, file: 'audio/narration-09.wav'},
];

export const soundCues = [
  {from: 15, file: 'audio/click.wav', volume: 0.12},
  {from: 29, file: 'audio/rise.wav', volume: 0.13},
  {from: 52, file: 'audio/pulse.wav', volume: 0.12},
  {from: 78, file: 'audio/error.wav', volume: 0.1},
  {from: 104, file: 'audio/click.wav', volume: 0.1},
  {from: 151, file: 'audio/pulse.wav', volume: 0.12},
  {from: 169, file: 'audio/rise.wav', volume: 0.14},
];
