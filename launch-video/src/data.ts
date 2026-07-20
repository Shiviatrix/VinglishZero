export const FPS = 30;
// The layout is authored on a stable design canvas and rendered at 4K.
export const DESIGN_WIDTH = 1920;
export const DESIGN_HEIGHT = 1080;
export const WIDTH = DESIGN_WIDTH * 2;
export const HEIGHT = DESIGN_HEIGHT * 2;
// Keep the rendered film safely below the Build Week three-minute limit.
export const DURATION = 179 * FPS;

export const COLORS = {
  background: '#0D0C0B',
  surface: '#E7E1D8',
  surfaceRaised: '#C8C0B6',
  text: '#F5F0E8',
  ink: '#191612',
  muted: '#B4AEA4',
  faint: '#7B756C',
  blue: '#A6C7D7',
  cyan: '#9BC6B8',
  violet: '#D7A9B1',
  green: '#B8C991',
  amber: '#E38A36',
  red: '#D17C62',
} as const;

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
    'function sum_positive(number count)',
    'returns number',
    'begin',
    '    let mutable total be 0',
    '    let mutable index be 0',
    '    repeat while index < count',
    '    begin',
    '        if index > 0',
    '            total += index',
    '        index += 1',
    '    end',
    '    return total',
    'end',
  ],
};

export const codeColors: Record<CodeLanguage, string> = {
  PYTHON: COLORS.blue,
  JAVA: COLORS.amber,
  C: COLORS.violet,
  VINGLISH: COLORS.cyan,
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
