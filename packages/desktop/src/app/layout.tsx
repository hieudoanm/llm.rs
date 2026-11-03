import type { Metadata } from 'next';
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

export const metadata: Metadata = {
  title: 'llm-client',
  description: 'Upload a CSV, choose a supervised model, download ONNX.',
};

const RootLayout = ({ children }: { children: React.ReactNode }) => (
  <html
    lang='en'
    data-theme='skl-dark'
    suppressHydrationWarning
    className={`${inter.variable} ${jetbrainsMono.variable} h-full antialiased`}>
    <body className='flex min-h-full flex-col'>
      <Providers>
        <Navbar />
        <main className='flex-1'>{children}</main>
        <Footer />
      </Providers>
    </body>
  </html>
);

export default RootLayout;
