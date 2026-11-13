import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import type { FC, ReactNode } from 'react';
import Footer from '@/components/molecules/Footer';
import Navbar from '@/components/molecules/Navbar';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

const GITHUB = 'https://github.com/hieudoanm/llm.rs';
const LOGO =
  process.env.NODE_ENV === 'production' ? '/llm.rs/logo.svg' : '/logo.svg';

const NAV_LINKS = [
  { label: 'Features', href: '#features' },
  { label: 'Demo', href: '#demo' },
  { label: 'How it works', href: '#how-it-works' },
  { label: 'Install', href: '#install' },
  { label: 'Docs', href: '/docs/' },
];

const DESCRIPTION =
  'Train supervised, semi-supervised, unsupervised, and offline reinforcement-learning models from CSV files. Export portable ONNX — all on your own machine.';

export const metadata: Metadata = {
  metadataBase: new URL('https://hieudoanm.github.io/llm.rs/'),
  title: {
    default: 'llm.rs — Machine Learning in Rust',
    template: '%s — llm.rs',
  },
  description: DESCRIPTION,
  openGraph: {
    siteName: 'llm.rs',
    type: 'website',
    title: 'llm.rs — Machine Learning in Rust',
    description: DESCRIPTION,
    url: 'https://hieudoanm.github.io/llm.rs/',
    images: ['/og.png'],
  },
  twitter: { card: 'summary_large_image', images: ['/og.png'] },
  icons: {
    icon: [
      { url: '/favicon.ico', sizes: 'any' },
      { url: '/favicon.svg', type: 'image/svg+xml' },
    ],
  },
};

interface RootLayoutProps {
  children: ReactNode;
}

const RootLayout: FC<RootLayoutProps> = ({ children }) => (
  <html
    lang="en"
    data-theme="llmrs"
    className={`scroll-smooth antialiased motion-reduce:scroll-auto ${inter.className}`}>
    <body className="bg-base-100 text-base-content m-0 leading-relaxed">
      <Navbar
        name="llm.rs"
        logoSrc={LOGO}
        links={NAV_LINKS}
        githubUrl={GITHUB}
      />
      {children}
      <Footer
        name="llm.rs"
        tagline="Local ML studio · MIT licensed"
        githubUrl={GITHUB}
      />
    </body>
  </html>
);

export default RootLayout;
