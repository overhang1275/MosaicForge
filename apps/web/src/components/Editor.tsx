'use client';

import { useEffect, useRef } from 'react';
import { useEditor } from '../editor/use-editor';
import { Inspector } from './Inspector';
import { Gallery, Preview } from './Results';
import { PdfExport } from './PdfExport';

export function Editor() {
  const context = useEditor();
  const errorRef = useRef<HTMLParagraphElement>(null);
  useEffect(() => {
    if (context.state.error) errorRef.current?.focus();
  }, [context.state.error]);
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
          {context.state.error && (
            <p
              className="error-message"
              role="alert"
              ref={errorRef}
              tabIndex={-1}
            >
              {context.state.error}
            </p>
          )}
          <div className="feedback" aria-live="polite" aria-atomic="true">
            <p>{context.state.message}</p>
            {context.state.downloading && <p>Preparando descarga…</p>}
          </div>
          <PdfExport {...context} />
          <Preview {...context} />
        </div>
        <Gallery {...context} />
      </main>
    </>
  );
}
