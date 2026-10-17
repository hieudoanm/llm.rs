import { cn } from '@/lib/cn';
import { API_BASE } from '@/lib/api';
import { PiDownload, PiFileZip } from 'react-icons/pi';
import { type Artefact } from '../_lib/types';

interface JobDownloadsProps {
  artefacts: Artefact[];
  jobId: string;
  isTerminal: boolean;
}

export const JobDownloads = ({ artefacts, jobId, isTerminal }: JobDownloadsProps) => {
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
            <a
              key={artefact.artefactId}
              href={`${API_BASE}/api/v1/jobs/${jobId}/artefacts/${artefact.artefactId}`}
              download={artefact.filename}
              className={className}>
              <span>{artefact.filename}</span>
              <PiDownload className='icon-sm' aria-hidden='true' />
            </a>
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
          <a
            href={`${API_BASE}/api/v1/jobs/${jobId}/artefacts.zip`}
            download
            className='btn btn-outline flex items-center justify-between'>
            <span>Download all</span>
            <PiFileZip className='icon-sm' aria-hidden='true' />
          </a>
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
