const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';
const eyebrow =
  'flex items-center gap-[9px] text-[11px] font-bold uppercase tracking-[.18em] text-orange-2';
const heading =
  'mt-5 mb-[18px] text-[clamp(38px,5vw,62px)] font-[620] leading-[1.02] tracking-[-.065em] max-[520px]:text-[42px]';

const tools = [
  [
    'M',
    '',
    'CANONICAL',
    'Desktop app',
    'Upload a CSV, configure a task, watch training, and download ONNX in a native workspace.',
    'Explore downloads',
    '/downloads',
  ],
  [
    '>_',
    'bg-[#285541] font-mono text-base text-[#9be0bc]',
    'HEADLESS',
    'Command line',
    'Register datasets, train models, inspect jobs, and export artefacts from a terminal.',
    'Read the CLI guide',
    '/docs/02-cli',
  ],
  [
    '◇',
    'bg-[#384a75] text-2xl text-[#b1caff]',
    'PORTABLE',
    'ONNX runtime',
    'Load the exported model in Python, JavaScript, Rust, or any runtime with ONNX support.',
    'Read the workflow',
    '/docs/03-workflow',
  ],
];

function ToolCard({
  mark,
  markClass,
  tag,
  title,
  body,
  link,
  href,
}: {
  mark: string;
  markClass: string;
  tag: string;
  title: string;
  body: string;
  link: string;
  href: string;
}) {
  return (
    <article className='border-line rounded-xl border bg-[#111a25] p-[25px]' key={title}>
      <div className='mb-10 flex items-center justify-between'>
        <span
          className={`grid h-[38px] w-[38px] place-items-center rounded-[9px] bg-[#b9523d] font-[750] text-[#fff3e7] ${markClass}`}>
          {mark}
        </span>
        <span className='text-[9px] font-bold tracking-[.15em] text-[#6e7c8f]'>{tag}</span>
      </div>
      <h3 className='m-0 text-xl'>{title}</h3>
      <p className='min-h-[76px] text-[13px]'>{body}</p>
      <a className='text-orange-2 text-xs font-[650]' href={href}>
        {link} <span className='ml-2'>→</span>
      </a>
    </article>
  );
}

export default function Tools() {
  return (
    <section className={`${sectionShell} py-[120px] pb-[150px] max-[520px]:py-[90px]`}>
      <div className='mx-auto max-w-[590px] text-center'>
        <div className={`${eyebrow} justify-center`}>Meet llm.rs where you work</div>
        <h2 className={heading}>
          One engine.{' '}
          <em className='text-orange-2 font-serif font-normal'>Your choice of surface.</em>
        </h2>
      </div>
      <div className='mt-[58px] grid grid-cols-3 gap-4 max-[850px]:grid-cols-1'>
        {tools.map(([mark, markClass, tag, title, body, link, href]) => (
          <ToolCard
            key={title}
            mark={mark}
            markClass={markClass}
            tag={tag}
            title={title}
            body={body}
            link={link}
            href={href}
          />
        ))}
      </div>
    </section>
  );
}
