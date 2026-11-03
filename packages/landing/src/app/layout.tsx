import type { Metadata } from 'next';
import type { ReactNode } from 'react';
import { siteDescription, siteName } from '../lib/site';
import './globals.css';

export const metadata: Metadata = {
  metadataBase: new URL(process.env.NEXT_PUBLIC_SITE_URL ?? 'http://localhost:4173'),
  title: {
    default: `${siteName} — Machine Learning in Rust`,
    template: `%s | ${siteName}`,
  },
  description: siteDescription,
  manifest: '/manifest.json',
  icons: {
    icon: '/images/svg/llm-logo.svg',
  },
  openGraph: {
    title: `${siteName} — Machine Learning in Rust`,
    description: siteDescription,
    images: ['/images/png/llm-og.png'],
  },
};

export default function RootLayout({ children }: Readonly<{ children: ReactNode }>) {
  return (
    <html lang='en'>
      <body>{children}</body>
    </html>
  );
}
