'use client';

import { cn } from '@/lib/cn';
import { PiDownload, PiFileZip } from 'react-icons/pi';
import { tauriDownloadArtefact } from '@/lib/tauri';
import { type Artefact } from '../_lib/types';

interface JobDownloadsProps {
  artefacts: Artefact[];
  jobId: string;
  isTerminal: boolean;
}

export const JobDownloads = ({ artefacts, jobId, isTerminal }: JobDownloadsProps) => {
  const handleDownload = async (artefactId: string, filename: string) => {
    try {
      const base64 = await tauriDownloadArtefact(jobId, artefactId);
      const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
      const blob = new Blob([bytes]);
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
    } catch {
      // Download failed silently; the error boundary handles it.
    }
  };

  const handleDownloadAll = async () => {
    for (const artefact of artefacts) {
      await handleDownload(artefact.artefactId, artefact.filename);
    }
  };

  return (
    <section className='mb-8'>
      <h2 className='text-text-primary mb-4 text-lg font-semibold'>Downloads</h2>
      <div className='grid gap-2 sm:max-w-md'>
        {artefacts.map((artefact) => {
          const className = cn(
            'btn flex items-center justify-between',
            artefact.format === 'onnx' ? 'btn-primary' : 'btn-ghost',
          );
          return isTerminal ? (
            <button
              key={artefact.artefactId}
              onClick={() => handleDownload(artefact.artefactId, artefact.filename)}
              className={className}>
              <span>{artefact.filename}</span>
              <PiDownload className='icon-sm' aria-hidden='true' />
            </button>
          ) : (
            <span
              key={artefact.artefactId}
              className={cn(className, 'btn-disabled')}
              aria-disabled='true'>
              <span>{artefact.filename}</span>
              <PiDownload className='icon-sm' aria-hidden='true' />
            </span>
          );
        })}
        {artefacts.length > 0 && isTerminal && (
          <button
            onClick={handleDownloadAll}
            className='btn btn-outline flex items-center justify-between'>
            <span>Download all</span>
            <PiFileZip className='icon-sm' aria-hidden='true' />
          </button>
        )}
        {artefacts.length === 0 && (
          <p className='text-text-secondary text-sm'>
            {isTerminal
              ? 'No artefacts were produced for this job.'
              : 'Artefacts will be available when training completes.'}
          </p>
        )}
      </div>
    </section>
  );
};
