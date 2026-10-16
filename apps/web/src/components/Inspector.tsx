import { useId } from 'react';
import type { EditorSettings, RangeName } from '../editor/model';
import type { useEditor } from '../editor/use-editor';
import { Field, Section } from './Fields';

type EditorContext = ReturnType<typeof useEditor>;
const ranges: { name: RangeName; label: string }[] = [
  { name: 'translateX', label: 'Traslación X' },
  { name: 'translateY', label: 'Traslación Y' },
  { name: 'rotationDegrees', label: 'Rotación (°)' },
  { name: 'scaleX', label: 'Escala X' },
  { name: 'scaleY', label: 'Escala Y' },
  { name: 'opacity', label: 'Opacidad' },
];

export function Inspector({ state, editor }: EditorContext) {
  const fileId = useId();
  const set = (patch: Partial<EditorSettings>) => editor.updateSettings(patch);
  return (
    <aside className="inspector" aria-label="Plantilla y ajustes">
      <div className="inspector-scroll">
        <Section title="Plantilla SVG" open>
          <div className="file-picker">
            <svg viewBox="0 0 32 32" fill="none" aria-hidden="true">
              <path d="M9 3h10l6 6v20H7V3h2Zm9 0v8h7" />
            </svg>
            <label htmlFor={fileId}>Selecciona un archivo SVG</label>
            <p>Hasta 5 MiB. Se enviará al importar.</p>
            <input
              id={fileId}
              name="template"
              autoComplete="off"
              aria-label="Archivo SVG"
              type="file"
              accept=".svg,image/svg+xml"
              onChange={(event) =>
                editor.chooseFile(event.target.files?.[0] ?? null)
              }
            />
            <p className="filename">
              {state.candidateName ?? 'Ningún archivo seleccionado'}
            </p>
          </div>
          <button
            className="button secondary full"
            disabled={!state.candidateName || state.isInspecting}
            onClick={() => void editor.importTemplate()}
          >
            {state.isInspecting ? 'Importando…' : 'Importar plantilla'}
          </button>
          {state.template && (
            <div className="template-summary">
              <strong>{state.template.name}</strong>
              <span>
                {state.template.metadata.width} ×{' '}
                {state.template.metadata.height} ·{' '}
                {state.template.metadata.layerIds.length} capas
              </span>
              <details>
                <summary>Identidad de la plantilla</summary>
                <p className="hash">{state.template.metadata.templateHash}</p>
                <p>
                  Motor {state.template.metadata.engineVersion} · Reglas{' '}
                  {state.template.metadata.rulesVersion}
                </p>
              </details>
            </div>
          )}
        </Section>
        <Section title="Reproducción" open>
          <Field
            label="Semilla"
            value={state.settings.seed}
            decimal
            onChange={(seed) => set({ seed })}
          />
          <Field
            label="Índice inicial"
            value={state.settings.startIndex}
            decimal
            onChange={(startIndex) => set({ startIndex })}
          />
          <Field
            label="Variantes de muestra"
            value={state.settings.count}
            min={1}
            max={12}
            step="1"
            onChange={(count) => set({ count })}
          />
          <p className="field-note">
            Semilla e índice: enteros decimales de 64 bits.
          </p>
          <p className="field-note">
            Muestra de hasta 12 variantes. La generación masiva aún no está
            disponible en esta vista.
          </p>
        </Section>
        <Section title="Salida" open>
          <div className="field-pair">
            <Field
              label="Ancho (px)"
              value={state.settings.width}
              min={1}
              max={512}
              step="1"
              onChange={(width) => set({ width })}
            />
            <Field
              label="Alto (px)"
              value={state.settings.height}
              min={1}
              max={512}
              step="1"
              onChange={(height) => set({ height })}
            />
          </div>
          <p className="field-note">
            De 1 a 512 px por eje en la vista previa.
          </p>
          <label className="field">
            <span>Fondo</span>
            <input
              type="text"
              name="background"
              autoComplete="off"
              value={state.settings.background}
              onChange={(event) => set({ background: event.target.value })}
              placeholder="#FFFFFF"
              spellCheck={false}
            />
          </label>
        </Section>
        <Section title="Transformaciones">
          {ranges.map(({ name, label }) => (
            <fieldset className="range-fields" key={name}>
              <legend>{label}</legend>
              <div className="field-pair">
                <Field
                  label="Mínimo"
                  value={state.settings.ranges[name].min}
                  onChange={(min) =>
                    set({
                      ranges: {
                        ...state.settings.ranges,
                        [name]: { ...state.settings.ranges[name], min },
                      },
                    })
                  }
                />
                <Field
                  label="Máximo"
                  value={state.settings.ranges[name].max}
                  onChange={(max) =>
                    set({
                      ranges: {
                        ...state.settings.ranges,
                        [name]: { ...state.settings.ranges[name], max },
                      },
                    })
                  }
                />
              </div>
            </fieldset>
          ))}
          <label className="check-field">
            <input
              type="checkbox"
              name="mirrorX"
              checked={state.settings.mirrorX}
              onChange={(event) => set({ mirrorX: event.target.checked })}
            />
            Espejo horizontal
          </label>
          <label className="check-field">
            <input
              type="checkbox"
              name="mirrorY"
              checked={state.settings.mirrorY}
              onChange={(event) => set({ mirrorY: event.target.checked })}
            />
            Espejo vertical
          </label>
        </Section>
        <Section title="Color y composición">
          <Field
            label="Distorsión"
            value={state.settings.distortion}
            min={0}
            onChange={(distortion) => set({ distortion })}
          />
          <label className="field">
            <span>Paleta opcional</span>
            <textarea
              name="palette"
              autoComplete="off"
              rows={3}
              value={state.settings.palette}
              onChange={(event) => set({ palette: event.target.value })}
              placeholder="#A736FF, #00D3EA"
              spellCheck={false}
            />
            <small>
              Colores hexadecimales separados por comas. Vacío conserva los
              colores originales.
            </small>
          </label>
          <label className="check-field">
            <input
              type="checkbox"
              name="shuffleLayers"
              checked={state.settings.shuffleLayers}
              onChange={(event) => set({ shuffleLayers: event.target.checked })}
            />
            Mezclar el orden de las capas
          </label>
          <label className="check-field">
            <input
              type="checkbox"
              name="cropEnabled"
              checked={state.settings.cropEnabled}
              onChange={(event) => set({ cropEnabled: event.target.checked })}
            />
            Activar recorte
          </label>
          {state.settings.cropEnabled && (
            <fieldset className="range-fields">
              <legend>Área de recorte</legend>
              <div className="field-pair">
                {(['x', 'y', 'width', 'height'] as const).map((key) => (
                  <Field
                    key={key}
                    label={
                      {
                        x: 'Posición X',
                        y: 'Posición Y',
                        width: 'Ancho',
                        height: 'Alto',
                      }[key]
                    }
                    value={state.settings.crop[key]}
                    onChange={(value) =>
                      set({ crop: { ...state.settings.crop, [key]: value } })
                    }
                  />
                ))}
              </div>
            </fieldset>
          )}
        </Section>
      </div>
      <div className="generate-actions">
        <button
          className="button secondary full"
          disabled={
            !state.template ||
            state.isInspecting ||
            state.isGenerating ||
            state.isExportingPdf ||
            state.downloading !== null
          }
          onClick={() => void editor.generate()}
        >
          {state.isGenerating ? 'Generando variantes…' : 'Generar variantes'}
        </button>
        {state.isGenerating && (
          <button
            className="button secondary full"
            onClick={() => editor.cancel()}
          >
            Cancelar generación
          </button>
        )}
      </div>
    </aside>
  );
}
