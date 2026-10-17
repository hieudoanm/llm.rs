import type { Metadata } from 'next';
import Link from 'next/link';
import type { ReactNode } from 'react';
import DocsSearch from '../../components/docs/DocsSearch';
import { getDocs } from '@/lib/docs';
import { repositoryUrl, basePath } from '@/lib/site';

export const metadata: Metadata = {
  title: 'Documentation',
  description:
    'Guides for training models from CSV with llm.rs, choosing task types and algorithms, and exporting ONNX.',
};

const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';

export default function DocsLayout({ children }: Readonly<{ children: ReactNode }>) {
  const docs = getDocs();

  return (
    <div className='flex min-h-screen flex-col'>
      <header className='relative z-[2] mx-auto flex w-[min(calc(100%_-_48px),1240px)] items-center justify-between py-7 max-[520px]:w-[min(calc(100%_-_32px),1240px)] max-[520px]:py-5'>
        <Link
          className='inline-flex items-center gap-2.5 text-[19px] font-[650] tracking-[-.03em]'
          href='/'
          aria-label='llm.rs home'>
          <img
            className='h-[30px] w-[30px] rounded-lg'
            src={`${basePath}/images/svg/llm-logo.svg`}
            alt=''
          />
          <span>llm.rs</span>
        </Link>
        <nav
          className='text-muted flex gap-[30px] text-[13px] max-[850px]:hidden'
          aria-label='Primary navigation'>
          <Link className='hover:text-ink' href='/'>
            Home
          </Link>
          <Link className='hover:text-ink' href='/#why-llm'>
            Why llm.rs
          </Link>
          <Link className='hover:text-ink' href='/#workflow'>
            Workflow
          </Link>
          <Link className='hover:text-ink' href='/docs'>
            Docs
          </Link>
        </nav>
        <a
          className='text-orange-2 hover:text-ink text-[13px] font-semibold max-[520px]:text-[11px]'
          href={repositoryUrl}>
          GitHub <span aria-hidden='true'>↗</span>
        </a>
      </header>

      <main
        className={`${sectionShell} grid flex-1 grid-cols-[230px_minmax(0,1fr)] items-start gap-12 py-14 pb-[90px] max-[900px]:grid-cols-1 max-[900px]:gap-[22px] max-[900px]:py-[34px] max-[900px]:pb-[70px]`}>
        <DocsSearch docs={docs} />
        <div className='border-line bg-panel/80 min-w-0 rounded-[22px] border px-12 py-[42px] max-[900px]:px-[22px] max-[900px]:py-7'>
          {children}
        </div>
      </main>

      <footer
        className={`${sectionShell} flex items-center justify-between gap-5 py-[30px] pb-[46px] text-[11px] text-[#6d7a8a] max-[520px]:flex-col max-[520px]:items-start`}>
        <Link
          className='inline-flex items-center gap-2.5 text-[19px] font-[650] tracking-[-.03em] text-[#d8dde5]'
          href='/'>
          <img className='h-6 w-6 rounded-lg' src={`${basePath}/images/svg/llm-logo.svg`} alt='' />
          <span>llm.rs</span>
        </Link>
        <span>CSV in. ONNX out. All on your machine.</span>
        <span>MIT licensed · Built with Rust</span>
      </footer>
    </div>
  );
}
