import type { Metadata } from 'next';
import Link from 'next/link';
import { notFound } from 'next/navigation';
import Markdown from '@/components/landing/Markdown';
import { getDoc, getDocs } from '@/lib/docs';

export const dynamicParams = false;

export function generateStaticParams() {
  return getDocs().map((doc) => ({ slug: doc.slug }));
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ slug: string }>;
}): Promise<Metadata> {
  const { slug } = await params;
  const doc = getDoc(slug);
  return doc ? { title: doc.title } : {};
}

export default async function DocPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const doc = getDoc(slug);
  if (!doc) notFound();

  const docs = getDocs();
  const currentIndex = docs.findIndex((d) => d.slug === slug);
  const previous = currentIndex > 0 ? docs[currentIndex - 1] : undefined;
  const next =
    currentIndex >= 0 && currentIndex < docs.length - 1 ? docs[currentIndex + 1] : undefined;

  return (
    <div>
      <Markdown content={doc.content} />
      <nav className='border-line mt-12 flex items-center justify-between gap-4 border-t pt-6'>
        <div className='flex-1'>
          {previous ? (
            <Link
              className='border-line text-ink hover:bg-panel-2 group flex items-center gap-2 rounded-lg border px-4 py-2 text-sm transition'
              href={`/docs/${previous.slug}`}>
              <span aria-hidden='true'>←</span>
              <span className='flex flex-col'>
                <span className='text-muted text-xs'>Previous</span>
                <span className='font-medium'>{previous.title}</span>
              </span>
            </Link>
          ) : null}
        </div>
        <div className='flex-1 text-right'>
          {next ? (
            <Link
              className='border-line text-ink hover:bg-panel-2 group ml-auto flex items-center justify-end gap-2 rounded-lg border px-4 py-2 text-sm transition'
              href={`/docs/${next.slug}`}>
              <span className='flex flex-col items-end'>
                <span className='text-muted text-xs'>Next</span>
                <span className='font-medium'>{next.title}</span>
              </span>
              <span aria-hidden='true'>→</span>
            </Link>
          ) : null}
        </div>
      </nav>
    </div>
  );
}
