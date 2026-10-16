'use client';

import { useEditor } from '../editor/use-editor';
import { Inspector } from './Inspector';
import { Gallery, Preview } from './Results';

export function Editor() {
  const context = useEditor();
  return (
    <>
      <a className="skip-link" href="#preview-title">
        Ir a la vista previa
      </a>
      <header className="studio-header">
        <span className="wordmark">MosaicForge</span>
        <span>Estudio de variantes</span>
      </header>
      <main className="studio">
        <Inspector {...context} />
        <div className="workspace">
          <Preview {...context} />
          <div className="feedback" aria-live="polite" aria-atomic="true">
            <p>{context.state.message}</p>
            {context.state.downloading && <p>Preparando descarga…</p>}
          </div>
          {context.state.error && (
            <p className="error-message" role="alert">
              {context.state.error}
            </p>
          )}
        </div>
        <Gallery {...context} />
      </main>
    </>
  );
}
