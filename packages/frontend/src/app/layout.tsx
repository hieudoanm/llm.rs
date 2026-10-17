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
const themeInitScript = `(()=>{try{var t=window.localStorage.getItem('skl-theme');if(t!=='skl-light'&&t!=='skl-dark'){t=window.matchMedia('(prefers-color-scheme: dark)').matches?'skl-dark':'skl-light';}document.documentElement.setAttribute('data-theme',t);}catch(e){}})();`;

export const metadata: Metadata = {
  title: 'scikit-learn-client',
  description: 'Upload a CSV, choose a supervised model, download ONNX.',
};

const RootLayout = ({ children }: { children: React.ReactNode }) => (
  <html
    lang='en'
    data-theme='skl-light'
    suppressHydrationWarning
    className={`${inter.variable} ${jetbrainsMono.variable} h-full antialiased`}>
    <body className='flex min-h-full flex-col'>
      <Script
        id='skl-theme-init'
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
