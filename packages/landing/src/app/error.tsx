'use client';

import Link from 'next/link';

export default function Error({
  error,
  reset,
}: Readonly<{
  error: Error;
  reset: () => void;
}>) {
  return (
    <div className='bg-bg text-ink flex min-h-screen flex-col items-center justify-center py-20'>
      <div className='max-w-[480px] text-center'>
        <h1 className='mb-2 text-[clamp(38px,7vw,62px)] leading-[1.02] font-[650] tracking-[-.075em]'>
          Something went wrong
        </h1>
        <p className='text-muted mb-6'>{error.message}</p>
        <div className='flex flex-col items-center gap-3 sm:flex-row'>
          <button
            className='border-orange bg-orange rounded-lg border px-[18px] py-[13px] text-xs font-[650] text-[#fff7ed] hover:bg-[#ff7955]'
            onClick={() => reset()}>
            Try again
          </button>
          <Link
            className='border-line bg-panel text-muted hover:bg-panel-2 hover:text-ink inline-flex items-center gap-2 rounded-lg border px-[18px] py-[13px] text-xs font-[650]'
            href='/docs'>
            Back to docs
          </Link>
        </div>
      </div>
    </div>
  );
}
