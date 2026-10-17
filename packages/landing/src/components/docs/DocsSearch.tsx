'use client';

import Link from 'next/link';
import { useState } from 'react';
import type { Doc } from '@/lib/docs';
import { repositoryUrl } from '@/lib/site';

const sidebarLink =
  'rounded-lg px-2.5 py-[7px] text-[13px] leading-[1.4] text-muted hover:bg-panel-2 hover:text-ink';

export default function DocsSearch({ docs }: Readonly<{ docs: Doc[] }>) {
  const [query, setQuery] = useState('');

  const filteredDocs = docs.filter(
    (doc) =>
      !query ||
      doc.title.toLowerCase().includes(query.toLowerCase()) ||
      doc.description?.toLowerCase().includes(query.toLowerCase()),
  );

  return (
    <aside
      className='border-line bg-panel/80 sticky top-7 flex flex-col gap-1 rounded-[18px] border p-5 max-[900px]:static max-[900px]:flex-row max-[900px]:flex-wrap'
      aria-label='Documentation navigation'>
      <p className='text-faint m-0 mb-2.5 text-[10px] font-bold tracking-[.16em] uppercase max-[900px]:w-full'>
        Documentation
      </p>
      <div className='mb-3 flex flex-col gap-1'>
        <label className='text-muted text-[10px] font-bold uppercase'>Search</label>
        <input
          aria-label='Search docs'
          className='border-line bg-bg focus:border-orange rounded-lg border px-3 py-2 text-sm focus:outline-none'
          placeholder='Search by title or description'
          value={query}
          onChange={(event) => setQuery(event.currentTarget.value)}
        />
      </div>
      <Link className={sidebarLink} href='/docs'>
        Overview
      </Link>
      {filteredDocs.map((doc) => (
        <Link className={sidebarLink} href={`/docs/${doc.slug}`} key={doc.slug}>
          {doc.title}
        </Link>
      ))}
      <a className={sidebarLink} href={`${repositoryUrl}/tree/main/docs`}>
        Repository docs <span aria-hidden='true'>↗</span>
      </a>
    </aside>
  );
}
