import type { Metadata } from 'next';
import Script from 'next/script';
import { Inter, JetBrains_Mono } from 'next/font/google';
import '@/styles/globals.css';
import { Providers } from '@/components/providers/Providers';
import { Navbar } from '@/components/layout/Navbar';
import { Footer } from '@/components/layout/Footer';

const inter = Inter({
  subsets: ['latin'],
  variable: '--font-inter',
  display: 'swap',
});

const jetbrainsMono = JetBrains_Mono({
  subsets: ['latin'],
  variable: '--font-jetbrains-mono',
  display: 'swap',
});

// Runs before paint so the stored / system theme is applied without a flash.
const themeInitScript = `(()=>{try{var t=window.localStorage.getItem('llm-theme');if(t!=='llm-light'&&t!=='llm-dark'){t=window.matchMedia('(prefers-color-scheme: dark)').matches?'llm-dark':'llm-light';}document.documentElement.setAttribute('data-theme',t);}catch(e){}})();`;

export const metadata: Metadata = {
  title: 'llm-client',
  description: 'Upload a CSV, choose a supervised model, download ONNX.',
};

const RootLayout = ({ children }: { children: React.ReactNode }) => (
  <html
    lang='en'
    data-theme='llm-light'
    suppressHydrationWarning
    className={`${inter.variable} ${jetbrainsMono.variable} h-full antialiased`}>
    <body className='flex min-h-full flex-col'>
      <Script
        id='llm-theme-init'
        strategy='beforeInteractive'
        dangerouslySetInnerHTML={{ __html: themeInitScript }}
      />
      <Providers>
        <Navbar />
        <main className='flex-1'>{children}</main>
        <Footer />
      </Providers>
    </body>
  </html>
);

export default RootLayout;
