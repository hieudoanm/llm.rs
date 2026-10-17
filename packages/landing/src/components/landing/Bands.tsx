const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';

export default function Bands() {
  return (
    <section className='border-line border-y bg-[#0e151f]'>
      <div
        className={`${sectionShell} text-faint flex min-h-[74px] items-center justify-between text-[11px] tracking-[.08em] uppercase max-[520px]:block max-[520px]:py-[18px]`}>
        <span className='max-[520px]:max-w-[130px]'>
          Made for people who train on their own machine
        </span>
        <div className='flex gap-7 font-bold text-[#77869a] max-[850px]:gap-2.5 max-[850px]:text-[9px] max-[520px]:mt-3 max-[520px]:justify-between'>
          <span>DATA</span>
          <span>MODELS</span>
          <span>METRICS</span>
          <span>ONNX</span>
        </div>
      </div>
    </section>
  );
}
