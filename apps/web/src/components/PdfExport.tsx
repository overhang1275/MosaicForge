import type { useEditor } from '../editor/use-editor';
import { Field } from './Fields';

export function PdfExport({ state, editor }: ReturnType<typeof useEditor>) {
  const ready =
    state.cards.length > 0 &&
    state.cards.every((card) => card.status === 'accepted');
  return (
    <section
      className="pdf-export"
      aria-label="PDF vectorial de la serie completa"
    >
      <div className="pdf-export-action">
        <button
          className="button primary"
          disabled={
            !ready ||
            state.isGenerating ||
            state.isInspecting ||
            state.isExportingPdf ||
            state.downloading !== null
          }
          onClick={() => void editor.exportPdf()}
        >
          {state.isExportingPdf
            ? 'Preparando PDF vectorial…'
            : 'Descargar PDF multipágina'}
        </button>
        <p>
          {state.cards.length > 0
            ? ready
              ? `Un PDF vectorial · ${state.cards.length} ${state.cards.length === 1 ? 'página' : 'páginas'} · serie completa guardada.`
              : `Serie de ${state.cards.length} ${state.cards.length === 1 ? 'variante' : 'variantes'} · PDF disponible cuando todas estén aceptadas.`
            : 'Un solo PDF vectorial con la serie completa de hasta 12 variantes.'}
        </p>
      </div>
      {!ready && state.cards.length > 0 && (
        <p className="field-note">
          Todas las variantes deben estar aceptadas. El PDF no omite páginas
          rechazadas o con error.
        </p>
      )}
      <details className="pdf-size">
        <summary>
          Tamaño del PDF · {state.pdfSettings.trimWidthMm} ×{' '}
          {state.pdfSettings.trimHeightMm} mm
        </summary>
        <div className="pdf-size-fields">
          <Field
            label="Ancho final (mm)"
            value={state.pdfSettings.trimWidthMm}
            min={1}
            max={2000}
            onChange={(trimWidthMm) =>
              editor.updatePdfSettings({ trimWidthMm })
            }
          />
          <Field
            label="Alto final (mm)"
            value={state.pdfSettings.trimHeightMm}
            min={1}
            max={2000}
            onChange={(trimHeightMm) =>
              editor.updatePdfSettings({ trimHeightMm })
            }
          />
          <Field
            label="Sangrado (mm)"
            value={state.pdfSettings.bleedMm}
            min={0}
            max={50}
            onChange={(bleedMm) => editor.updatePdfSettings({ bleedMm })}
          />
        </div>
        <p className="field-note">
          Tamaño final por página. El sangrado se añade por fuera; estos ajustes
          se aplican al exportar.
        </p>
      </details>
    </section>
  );
}
