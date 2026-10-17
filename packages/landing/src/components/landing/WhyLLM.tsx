const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';
const eyebrow =
  'flex items-center gap-[9px] text-[11px] font-bold uppercase tracking-[.18em] text-orange-2';
const heading =
  'mt-5 mb-[18px] text-[clamp(38px,5vw,62px)] font-[620] leading-[1.02] tracking-[-.065em] max-[520px]:text-[42px]';

const features = [
  {
    large: true,
    icon: '✦',
    iconClass: '',
    title: 'Six task types, one workflow',
    body: 'Classification, regression, semi-supervised self-training, clustering, anomaly detection, and offline reinforcement learning share the same upload → configure → train → export path.',
  },
  {
    icon: '◈',
    iconClass: 'bg-[#36755d42] text-[#83d6ae]',
    title: 'Local-first and private',
    body: 'Your CSV and trained artefacts stay on your machine. llm.rs persists datasets, jobs, metrics, and models to local SQLite storage.',
    footnote: 'No cloud account required',
  },
  {
    icon: '▣',
    iconClass: 'bg-[#8b543342] text-[#ffc07b]',
    title: 'ONNX is the contract',
    body: 'Every successful job exports a portable .onnx model that runs in other runtimes and languages, independent of the training code.',
    footnote: 'Portable by design',
  },
];

function FeatureCard({ feature }: { feature: (typeof features)[number] }) {
  return (
    <article
      className={`border-line relative min-h-[320px] overflow-hidden rounded-[13px] border bg-[linear-gradient(145deg,#151f2b,#101720)] p-7 ${feature.large ? 'max-[850px]:min-h-[380px]' : ''}`}
      key={feature.title}>
      <div
        className={`mb-[35px] grid h-10 w-10 place-items-center rounded-[10px] bg-[#e25e4750] text-xl text-white ${feature.iconClass}`}>
        {feature.icon}
      </div>
      <h3 className='m-0 mb-3 text-xl tracking-[-.04em]'>{feature.title}</h3>
      <p className='m-0 text-[13px]'>{feature.body}</p>
      {feature.title === 'Six task types, one workflow' ? (
        <div className='absolute right-[22px] bottom-[-2px] left-[22px] rounded-t-lg border border-b-0 border-[#304052] bg-[#0d141d] px-[18px] pt-[17px] pb-3 font-mono text-[11px] leading-[1.9]'>
          <div className='mb-[5px] whitespace-nowrap text-[#c4cbd5]'># Train a classifier</div>
          <div className='whitespace-nowrap'>
            <span className='inline-block w-[25px] text-[#566578]'>01</span>
            <span className='text-[#d39cff]'>llm</span> dataset{' '}
            <span className='text-[#f5986d]'>demo.csv</span>
          </div>
          <div className='whitespace-nowrap'>
            <span className='inline-block w-[25px] text-[#566578]'>02</span>
            <span className='text-[#d39cff]'>llm</span> train{' '}
            <span className='text-[#77b7ff]'>--task</span> classification
          </div>
          <div className='mt-[5px] border-t border-[#263341] pt-[5px] text-[#72d3a2]'>
            ↳ model.onnx exported
          </div>
        </div>
      ) : (
        <div className='absolute bottom-[25px] flex items-center gap-2 text-[11px] text-[#7f8c9c]'>
          <span className='bg-mint h-1.5 w-1.5 rounded-full' /> {feature.footnote}
        </div>
      )}
    </article>
  );
}

export default function WhyLLM() {
  return (
    <section className={`${sectionShell} py-[145px] max-[520px]:py-[90px]`} id='why-llm'>
      <div className='max-w-[590px]'>
        <div className={eyebrow}>One workflow. Every learning mode.</div>
        <h2 className={heading}>
          From raw CSV
          <br />
          to a <em className='text-orange-2 font-serif font-normal'>portable model.</em>
        </h2>
        <p className='max-w-[470px] text-base'>
          llm.rs brings upload, mapping, training, metrics, and export together in a local-first
          studio. No cloud account, no notebooks, no hidden pipeline.
        </p>
      </div>
      <div className='mt-[55px] grid grid-cols-[1.3fr_1fr_1fr] gap-4 max-[850px]:grid-cols-1'>
        {features.map((feature) => (
          <FeatureCard key={feature.title} feature={feature} />
        ))}
      </div>
    </section>
  );
}
