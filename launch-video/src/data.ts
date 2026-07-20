export const FPS = 30;
export const WIDTH = 1920;
export const HEIGHT = 1080;
export const DURATION = 179 * FPS + 27;

export const COLORS = {
  background: '#24231F',
  surface: '#F2E8D8',
  surfaceRaised: '#E2D1B8',
  grid: '#675F53',
  text: '#F7F0E5',
  ink: '#28241F',
  muted: '#BFAF98',
  faint: '#8B7D6A',
  blue: '#B08A5A',
  cyan: '#2E706C',
  violet: '#BD7A1C',
  green: '#4B7453',
  amber: '#C88722',
  red: '#A84A36',
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
    'function filter_map_reduce(values) -> integer {',
    '    mutable result: integer = 0',
    '    while has_more {',
    '        if value > 0 {',
    '            result = combine(',
    '                result, transform(value))',
    '        }',
    '    } return result',
    '}',
  ],
};

export const codeColors: Record<CodeLanguage, string> = {
  PYTHON: '#5ea9ff',
  JAVA: '#f5bd57',
  C: '#a78bfa',
  VINGLISH: '#69e4dc',
};

export const cueFrames = {
  hook: 0,
  languages: 8 * FPS,
  reveal: 30 * FPS,
  engine: 55 * FPS,
  diagnostics: 95 * FPS,
  composition: 115 * FPS,
  determinism: 135 * FPS,
  depth: 150 * FPS,
  ending: 165 * FPS,
} as const;

export const narrationTracks = [
  {from: 0, file: 'audio/narration-01.wav'},
  {from: 8, file: 'audio/narration-02.wav'},
  {from: 30, file: 'audio/narration-03.wav'},
  {from: 43, file: 'audio/narration-04.wav'},
  {from: 55, file: 'audio/narration-05.wav'},
  {from: 95, file: 'audio/narration-06.wav'},
  {from: 115, file: 'audio/narration-07.wav'},
  {from: 135, file: 'audio/narration-08.wav'},
  {from: 150, file: 'audio/narration-09.wav'},
  {from: 165, file: 'audio/narration-10.wav'},
];

export const soundCues = [
  {from: 8.0, file: 'audio/click.wav', volume: 0.16},
  {from: 30.0, file: 'audio/rise.wav', volume: 0.2},
  {from: 43.0, file: 'audio/pulse.wav', volume: 0.16},
  {from: 55.0, file: 'audio/rise.wav', volume: 0.16},
  {from: 95.0, file: 'audio/error.wav', volume: 0.12},
  {from: 115.0, file: 'audio/pulse.wav', volume: 0.16},
  {from: 135.0, file: 'audio/click.wav', volume: 0.16},
  {from: 165.0, file: 'audio/rise.wav', volume: 0.16},
];
