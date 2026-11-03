const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';

export default function Footer() {
  return (
    <footer
      className={`${sectionShell} flex items-center justify-between gap-5 py-[30px] text-[11px] text-[#6d7a8a] max-[520px]:flex-col max-[520px]:items-start max-[520px]:py-[25px]`}>
      <a
        className='inline-flex items-center gap-2.5 text-[19px] font-[650] tracking-[-.03em] text-[#d8dde5]'
        href='/'>
        <img className='h-6 w-6 rounded-lg' src='/images/svg/llm-logo.svg' alt='' />
        <span>llm.rs</span>
      </a>
      <span>CSV in. ONNX out. All on your machine.</span>
      <span>MIT licensed · Built with Rust</span>
    </footer>
  );
}
