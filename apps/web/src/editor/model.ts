import type {
  GenerationSpec,
  InspectResponse,
  VariantResult,
} from '../../../../contracts/generated/typescript/preview';
export type RangeName =
  | 'translateX'
  | 'translateY'
  | 'rotationDegrees'
  | 'scaleX'
  | 'scaleY'
  | 'opacity';
export interface EditorSettings {
  seed: string;
  startIndex: string;
  count: string;
  width: string;
  height: string;
  background: string;
  ranges: Record<RangeName, { min: string; max: string }>;
  mirrorX: boolean;
  mirrorY: boolean;
  shuffleLayers: boolean;
  distortion: string;
  palette: string;
  cropEnabled: boolean;
  crop: { x: string; y: string; width: string; height: string };
}
export interface PreviewCard {
  index: string;
  status: 'pending' | 'generating' | 'accepted' | 'rejected' | 'error';
  url?: string;
  result?: VariantResult;
  error?: string;
}
export interface EditorState {
  candidateName: string | null;
  template: { name: string; svg: string; metadata: InspectResponse } | null;
  isInspecting: boolean;
  isGenerating: boolean;
  isExportingPdf: boolean;
  pdfSettings: PdfSettings;
  settings: EditorSettings;
  cards: PreviewCard[];
  generationStats: {
    requested: number;
    attempted: number;
    rejected: number;
  } | null;
  selectedIndex: string | null;
  error: string | null;
  message: string;
  downloading: string | null;
}
export interface PdfSettings {
  trimWidthMm: string;
  trimHeightMm: string;
  bleedMm: string;
}
export interface Snapshot {
  templateSvg: string;
  generation: GenerationSpec;
}
export const initialSettings = (): EditorSettings => ({
  seed: '2026',
  startIndex: '0',
  count: '9',
  width: '512',
  height: '512',
  background: 'transparent',
  ranges: {
    translateX: { min: '-12', max: '12' },
    translateY: { min: '-12', max: '12' },
    rotationDegrees: { min: '0', max: '360' },
    scaleX: { min: '1', max: '1.8' },
    scaleY: { min: '1', max: '1.8' },
    opacity: { min: '1', max: '1' },
  },
  mirrorX: true,
  mirrorY: true,
  shuffleLayers: false,
  distortion: '0',
  palette: '',
  cropEnabled: false,
  crop: { x: '0', y: '0', width: '100', height: '100' },
});
