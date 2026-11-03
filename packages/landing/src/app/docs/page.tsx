import { notFound } from 'next/navigation';
import Markdown from '@/components/Markdown';
import { getDoc } from '@/lib/docs';

export default function DocsPage() {
  const doc = getDoc('index');
  if (!doc) notFound();
  return <Markdown content={doc.content} />;
}
