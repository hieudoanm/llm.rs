import Link from 'next/link';
import { repositoryUrl, basePath } from '../../lib/site';

const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';
const eyebrow =
  'flex items-center gap-[9px] text-[11px] font-bold uppercase tracking-[.18em] text-orange-2';
const heading =
  'mt-5 mb-[18px] text-[clamp(38px,5vw,62px)] font-[620] leading-[1.02] tracking-[-.065em] max-[520px]:text-[42px]';
const buttonBase =
  'inline-flex items-center gap-[14px] rounded-lg border px-[18px] py-[13px] text-[13px] font-[650] transition-[transform,border-color,background] duration-200 hover:-translate-y-0.5 hover:border-[#d4654d]';
const buttonPrimary = `${buttonBase} border-orange bg-orange text-[#fff7ed] shadow-[0_10px_30px_#f06b4d25] hover:bg-[#ff7955]`;
const buttonQuiet = `${buttonBase} border-line bg-[#141d28] text-[#c8d0db]`;

export default function GetStarted() {
  return (
    <section
      className='relative overflow-hidden border-t border-[#283646] bg-[#151d28]'
      id='get-started'>
      <div className='absolute right-[12%] bottom-[-230px] h-[480px] w-[480px] rounded-full bg-[radial-gradient(circle,#d8564238,transparent_66%)]' />
      <div
        className={`${sectionShell} relative grid grid-cols-[100px_1fr_auto] items-center gap-[30px] py-[105px] max-[850px]:grid-cols-[75px_1fr] max-[520px]:grid-cols-1 max-[520px]:gap-5 max-[520px]:py-[75px]`}>
        <img
          className='h-[92px] w-[92px] drop-shadow-[0_15px_25px_#0006] max-[520px]:h-[70px] max-[520px]:w-[70px]'
          src={`${basePath}/images/svg/llm-logo.svg`}
          alt=''
        />
        <div>
          <div className={eyebrow}>Start with a CSV</div>
          <h2 className={`${heading} mb-3`}>
            Train on your machine.
            <br />
            <em className='text-orange-2 font-serif font-normal'>Export everywhere.</em>
          </h2>
          <p className='m-0 text-[13px]'>
            Open source. Rust-powered. Six task types. One portable ONNX artefact.
          </p>
        </div>
        <div className='flex flex-col items-stretch gap-[9px] max-[850px]:col-span-full max-[850px]:flex-row max-[520px]:col-auto max-[520px]:grid'>
          <a className={`${buttonPrimary} justify-center whitespace-nowrap`} href={repositoryUrl}>
            View the repository <span>↗</span>
          </a>
          <Link className={`${buttonQuiet} justify-center whitespace-nowrap`} href='/docs'>
            Read the docs <span>→</span>
          </Link>
        </div>
      </div>
    </section>
  );
}
