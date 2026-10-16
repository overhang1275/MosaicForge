import type { Metadata, Viewport } from 'next';
import './globals.css';
export const viewport: Viewport = { themeColor: '#080c18' };
export const metadata: Metadata = {
  title: 'MosaicForge',
  description: 'Variantes vectoriales reproducibles con motor CPU local',
};
export default function RootLayout({
  children,
}: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="es">
      <body>{children}</body>
    </html>
  );
}
