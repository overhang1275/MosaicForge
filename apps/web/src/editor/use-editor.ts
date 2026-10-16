'use client';
import { useEffect, useState, useSyncExternalStore } from 'react';
import { createEditor } from './controller';
import { browserPorts } from './client';
export function useEditor() {
  const [editor] = useState(() => createEditor(browserPorts));
  const state = useSyncExternalStore(
    editor.subscribe,
    editor.getSnapshot,
    editor.getSnapshot,
  );
  useEffect(() => () => editor.dispose(), [editor]);
  return { state, editor };
}
