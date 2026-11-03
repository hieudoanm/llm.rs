import Link from 'next/link';

export default function NotFound() {
  return (
    <div className='bg-bg text-ink flex min-h-screen flex-col items-center justify-center py-20'>
      <div className='max-w-[480px] text-center'>
        <img className='mx-auto mb-6 h-[48px] w-[48px]' src='/images/svg/llm-logo.svg' alt='' />
        <h1 className='mb-2 text-[clamp(38px,7vw,62px)] leading-[1.02] font-[650] tracking-[-.075em]'>
          Page not found
        </h1>
        <p className='text-muted mx-auto mb-8 max-w-[320px]'>
          The documentation page you are looking for does not exist.
        </p>
        <Link
          className='border-orange bg-orange inline-flex items-center gap-2 rounded-lg border px-[18px] py-[13px] text-xs font-[650] text-[#fff7ed] hover:bg-[#ff7955]'
          href='/docs'>
          Back to docs <span aria-hidden='true'>→</span>
        </Link>
      </div>
    </div>
  );
}
