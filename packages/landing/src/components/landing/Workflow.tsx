const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';
const eyebrow =
  'flex items-center gap-[9px] text-[11px] font-bold uppercase tracking-[.18em] text-orange-2';
const heading =
  'mt-5 mb-[18px] text-[clamp(38px,5vw,62px)] font-[620] leading-[1.02] tracking-[-.065em] max-[520px]:text-[42px]';

const workflow = [
  ['01', 'Upload', 'Register a CSV with a header row.'],
  ['02', 'Map', 'Choose features X and, when required, a label Y.'],
  ['03', 'Train', 'Pick a task and algorithm, then fit the model.'],
  ['04', 'Export', 'Download the ONNX artefact and its metrics.'],
];

export default function Workflow() {
  return (
    <section className='border-y border-[#1e2b38] bg-[#0d141e]' id='workflow'>
      <div className={`${sectionShell} py-[125px] pb-[135px] max-[520px]:py-[90px]`}>
        <div className='mx-auto max-w-[590px] text-center'>
          <div className={`${eyebrow} justify-center`}>From upload to artefact</div>
          <h2 className={heading}>
            A workflow that ends
            <br />
            <em className='text-orange-2 font-serif font-normal'>with a portable model.</em>
          </h2>
        </div>
        <div className='mt-[70px] flex items-stretch max-[850px]:grid max-[850px]:grid-cols-2 max-[850px]:gap-5 max-[520px]:grid-cols-1'>
          {workflow.map(([number, title, body], index) => (
            <div className='contents' key={title}>
              <div
                className={`flex-1 border-t px-[22px] py-5 ${index === 0 ? 'border-orange border-t-2' : 'border-[#2b3a4a]'}`}>
                <span className='text-orange mb-8 block font-mono text-xs'>{number}</span>
                <strong className='text-lg'>{title}</strong>
                <p className='mb-0 text-xs'>{body}</p>
              </div>
              {index < workflow.length - 1 && (
                <div className='mt-0 w-[18px] border-t border-dashed border-[#344455] max-[850px]:hidden' />
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
