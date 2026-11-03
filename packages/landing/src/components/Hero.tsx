import { repositoryUrl } from '../lib/site';

const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';
const eyebrow =
  'flex items-center gap-[9px] text-[11px] font-bold uppercase tracking-[.18em] text-orange-2';
const buttonBase =
  'inline-flex items-center gap-[14px] rounded-lg border px-[18px] py-[13px] text-[13px] font-[650] transition-[transform,border-color,background] duration-200 hover:-translate-y-0.5 hover:border-[#d4654d]';
const buttonPrimary = `${buttonBase} border-orange bg-orange text-[#fff7ed] shadow-[0_10px_30px_#f06b4d25] hover:bg-[#ff7955]`;
const buttonQuiet = `${buttonBase} border-line bg-[#141d28] text-[#c8d0db]`;

export default function Hero() {
  return (
    <section
      className={`${sectionShell} relative grid min-h-[640px] grid-cols-[1fr_.9fr] items-center gap-[50px] pt-20 pb-[110px] max-[850px]:grid-cols-1 max-[850px]:pt-[55px] max-[520px]:pt-[50px] max-[520px]:pb-[75px]`}>
      <div className='relative z-[1]'>
        <div className={eyebrow}>
          <span className='bg-orange h-[7px] w-[7px] rounded-full shadow-[0_0_15px_#f06b4d]' />{' '}
          Machine Learning in Rust
        </div>
        <h1 className='mt-6 mb-[22px] max-w-[670px] text-[clamp(54px,7vw,92px)] leading-[.96] font-[650] tracking-[-.075em] max-[850px]:text-[64px] max-[520px]:text-[52px]'>
          CSV in.
          <br />
          <em className='text-orange-2 font-serif font-normal'>ONNX out.</em>
        </h1>
        <p className='m-0 max-w-[535px] text-[18px] max-[520px]:text-[16px]'>
          Train supervised, semi-supervised, unsupervised, and offline reinforcement-learning models
          from tabular data — then export a portable model that runs anywhere. All on your own
          machine.
        </p>
        <div className='my-8 flex flex-wrap gap-3'>
          <a className={buttonPrimary} href='#get-started'>
            Train your first model <span aria-hidden='true'>→</span>
          </a>
          <a className={buttonQuiet} href={repositoryUrl}>
            View on GitHub <span aria-hidden='true'>↗</span>
          </a>
        </div>
        <div className='text-faint flex flex-wrap gap-[18px] text-[11px] max-[520px]:grid max-[520px]:gap-2'>
          <span className='flex items-center gap-1.5'>
            <span className='text-mint font-extrabold'>✓</span> Six task types
          </span>
          <span className='flex items-center gap-1.5'>
            <span className='text-mint font-extrabold'>✓</span> ONNX export
          </span>
          <span className='flex items-center gap-1.5'>
            <span className='text-mint font-extrabold'>✓</span> Local-first by default
          </span>
        </div>
      </div>
      <div
        className='relative grid h-[500px] place-items-center max-[850px]:h-[390px] max-[520px]:h-[330px]'
        aria-label='llm.rs training preview'>
        <div className='absolute h-[190px] w-[530px] rotate-[-25deg] rounded-full border border-[#d6745840] max-[520px]:w-[360px]' />
        <div className='absolute h-[140px] w-[450px] rotate-[36deg] rounded-full border border-[#8498ad30] max-[520px]:w-[300px]' />
        <div className='animate-float relative z-[1] h-[300px] w-[300px] drop-shadow-[0_28px_45px_#0008] max-[520px]:h-[220px] max-[520px]:w-[220px]'>
          <img className='h-full w-full' src='/images/svg/llm-logo.svg' alt='llm.rs logo' />
        </div>
        <div className='absolute top-[74px] right-0 z-[2] rounded-[9px] border border-[#3b4857] bg-[#141d28e8] px-4 py-[14px] font-mono text-xs leading-[1.6] shadow-[0_18px_40px_#0005] max-[520px]:top-6 max-[520px]:right-[-7px] max-[520px]:text-[9px]'>
          <span className='mb-2.5 flex gap-1'>
            <i className='h-[5px] w-[5px] rounded-full bg-[#c35f4a]' />
            <i className='h-[5px] w-[5px] rounded-full bg-[#d39b55]' />
            <i className='h-[5px] w-[5px] rounded-full bg-[#6eaf83]' />
          </span>
          <code>
            <b>llm</b> train <span className='text-[#f39467]'>demo.csv</span>
          </code>
        </div>
        <div className='absolute bottom-[72px] left-[2px] z-[2] rounded-[9px] border border-[#3b4857] bg-[#141d28e8] px-4 py-[14px] font-mono text-xs leading-[1.6] shadow-[0_18px_40px_#0005] max-[520px]:bottom-5 max-[520px]:left-[-7px] max-[520px]:text-[9px]'>
          <span className='mb-[5px] block text-[9px] tracking-[.12em] text-[#718096]'>
            OUT [ONNX]
          </span>
          <code>
            <span className='text-[#75d6a5]'>model.onnx</span>
          </code>
        </div>
        <span className='absolute top-1/4 left-[10%] text-[13px] text-[#ffb478]'>✦</span>
        <span className='absolute top-[35%] right-[7%] text-[9px] text-[#ffb478]'>✦</span>
        <span className='absolute right-[22%] bottom-[18%] text-[25px] text-[#ffb478]'>·</span>
      </div>
    </section>
  );
}
