/* eslint-disable @next/next/no-img-element -- Engine PNG Blob URLs are local, immutable previews; the Next image optimizer cannot fetch them. */
import type { PreviewCard } from '../editor/model';
import type { useEditor } from '../editor/use-editor';
import { ImageIcon } from './Fields';
import { rejectionCopy } from './rejection-copy';

type EditorContext = ReturnType<typeof useEditor>;
const statusLabels: Record<PreviewCard['status'], string> = {
  pending: 'Pendiente',
  generating: 'Generando',
  accepted: 'Aceptada',
  rejected: 'Rechazada',
  error: 'Error',
};

function Downloads({
  card,
  editor,
  downloading,
  isGenerating,
  isExportingPdf,
}: {
  card: PreviewCard;
  isGenerating: boolean;
  isExportingPdf: boolean;
  editor: EditorContext['editor'];
  downloading: string | null;
}) {
  return (
    <details className="secondary-downloads">
      <summary>SVG / PNG</summary>
      <div className="downloads">
        {(['svg', 'png'] as const).map((format) => (
          <button
            key={format}
            className="button secondary"
            disabled={downloading !== null || isGenerating || isExportingPdf}
            aria-label={`Descargar ${format.toUpperCase()} de la variante ${card.index}`}
            onClick={() => void editor.download(card.index, format)}
          >
            {format.toUpperCase()}
          </button>
        ))}
      </div>
    </details>
  );
}

export function Preview({ state, editor }: EditorContext) {
  const card = state.cards.find((item) => item.index === state.selectedIndex);
  return (
    <section className="preview-panel" aria-labelledby="preview-title">
      <div className="panel-heading">
        <h1 id="preview-title" tabIndex={-1}>
          Vista previa
        </h1>
        {card && (
          <span className={`status status-${card.status}`}>
            {statusLabels[card.status]}
          </span>
        )}
      </div>
      <div
        className={`preview-canvas ${card?.url ? 'has-artwork' : ''}`}
        aria-busy={card?.status === 'generating'}
      >
        {card?.url ? (
          <img
            src={card.url}
            width={512}
            height={512}
            decoding="async"
            alt={`Variante ${card.index} generada por el motor CPU`}
          />
        ) : (
          <div className="empty-state">
            <ImageIcon />
            <h2>
              {!card
                ? 'Tu próxima variante empieza aquí'
                : card.status === 'rejected'
                  ? 'Variante rechazada'
                  : card.status === 'error'
                    ? 'No se pudo generar esta variante'
                    : card.status === 'generating'
                      ? 'Generando tu variante…'
                      : 'Esta variante está en espera'}
            </h2>
            <p>
              {!card
                ? 'Importa una plantilla SVG y genera una primera serie.'
                : card.result?.reason
                  ? rejectionCopy(card.result.reason)
                  : (card.error ??
                    'Los resultados aparecerán al finalizar el cálculo.')}
            </p>
          </div>
        )}
      </div>
      <div className="preview-footer">
        <span>
          {card ? `Variante ${card.index}` : 'Sin variante seleccionada'}
        </span>
        {card?.status === 'accepted' && (
          <Downloads
            card={card}
            editor={editor}
            downloading={state.downloading}
            isGenerating={state.isGenerating}
            isExportingPdf={state.isExportingPdf}
          />
        )}
      </div>
      <details className="variant-metadata">
        <summary>Información de la variante</summary>
        {card?.result ? (
          <dl>
            <dt>Índice</dt>
            <dd>{card.result.index}</dd>
            <dt>Semilla</dt>
            <dd>{card.result.seed}</dd>
            <dt>Motor</dt>
            <dd>
              {card.result.engineVersion} · {card.result.backend} ·{' '}
              {card.result.device}
            </dd>
            <dt>Reglas</dt>
            <dd>{card.result.rulesVersion}</dd>
            {card.result.reason && (
              <>
                <dt>Código de rechazo</dt>
                <dd>{card.result.reason}</dd>
              </>
            )}
            <dt>Hash de plantilla</dt>
            <dd className="hash">{card.result.templateHash}</dd>
            <dt>Hash de configuración</dt>
            <dd className="hash">{card.result.configurationHash}</dd>
            {card.result.artifactHash && (
              <>
                <dt>Hash de artefacto PNG</dt>
                <dd className="hash">{card.result.artifactHash}</dd>
              </>
            )}
          </dl>
        ) : (
          <p>
            Selecciona un resultado para consultar su semilla y sus hashes de
            reproducción.
          </p>
        )}
      </details>
    </section>
  );
}

export function Gallery({ state, editor }: EditorContext) {
  const accepted = state.cards.filter(
    (card) => card.status === 'accepted',
  ).length;
  return (
    <section className="gallery" aria-labelledby="gallery-title">
      <div className="panel-heading">
        <h2 id="gallery-title">Variantes</h2>
        <span className="count" aria-label={`${accepted} variantes aceptadas`}>
          {accepted}
          {state.generationStats ? ` / ${state.generationStats.requested}` : ''}
        </span>
      </div>
      {state.generationStats && (
        <p className="field-note" aria-live="polite">
          {state.generationStats.attempted} intentos ·{' '}
          {state.generationStats.rejected} descartadas. Solo las aceptadas
          cuentan para la serie.
        </p>
      )}
      {state.cards.length === 0 ? (
        <div className="empty-state gallery-empty">
          <ImageIcon />
          <h3>Las variantes aparecerán aquí</h3>
          <p>Genera variantes para ver los resultados en esta galería.</p>
        </div>
      ) : (
        <div className="gallery-grid">
          {state.cards.map((card) => (
            <article
              className={`variant-card ${state.selectedIndex === card.index ? 'selected' : ''}`}
              key={card.index}
            >
              <button
                className="variant-select"
                aria-label={`Ver variante ${card.index}: ${statusLabels[card.status]}`}
                aria-pressed={state.selectedIndex === card.index}
                onClick={() => editor.select(card.index)}
              >
                <div className="thumbnail">
                  {card.url ? (
                    <img
                      src={card.url}
                      width={512}
                      height={512}
                      decoding="async"
                      alt=""
                    />
                  ) : (
                    <span className={`status status-${card.status}`}>
                      {statusLabels[card.status]}
                    </span>
                  )}
                </div>
                <span className="variant-label">Variante {card.index}</span>
                <span className={`status status-${card.status}`}>
                  {statusLabels[card.status]}
                </span>
              </button>
              {(card.result?.reason || card.error) && (
                <p className="card-reason">
                  {card.result?.reason
                    ? rejectionCopy(card.result.reason)
                    : card.error}
                </p>
              )}
              {card.status === 'accepted' && (
                <Downloads
                  card={card}
                  editor={editor}
                  downloading={state.downloading}
                  isGenerating={state.isGenerating}
                  isExportingPdf={state.isExportingPdf}
                />
              )}
            </article>
          ))}
        </div>
      )}
    </section>
  );
}
