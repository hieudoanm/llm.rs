const sectionShell =
  'mx-auto w-[min(calc(100%_-_48px),1160px)] max-[520px]:w-[min(calc(100%_-_32px),1160px)]';
const eyebrow =
  'flex items-center gap-[9px] text-[11px] font-bold uppercase tracking-[.18em] text-orange-2';
const heading =
  'mt-5 mb-[18px] text-[clamp(38px,5vw,62px)] font-[620] leading-[1.02] tracking-[-.065em] max-[520px]:text-[42px]';

export default function BuiltDifferent() {
  return (
    <section
      className={`${sectionShell} grid grid-cols-2 items-center gap-[100px] py-[150px] max-[850px]:grid-cols-1 max-[850px]:gap-[50px] max-[850px]:py-[100px]`}
      id='built-different'>
      <div className='relative min-h-[410px] pt-[38px]'>
        <div className='relative mx-auto -mb-[2px] w-[80%] translate-x-[38px] rounded-[9px] border border-[#354556] bg-[#1b2a38] px-6 py-[18px] shadow-[0_12px_25px_#0003]'>
          <span className='text-orange mb-[5px] block font-mono text-[9px] tracking-[.16em]'>
            DESKTOP
          </span>
          <strong className='block text-[17px]'>Next.js + Tauri UI</strong>
          <small className='text-[11px] text-[#738398]'>TypeScript · React</small>
        </div>
        <div className='relative mx-auto -mb-[2px] w-[80%] translate-x-[18px] rounded-[9px] border border-[#354556] bg-[#182634] px-6 py-[18px] shadow-[0_12px_25px_#0003]'>
          <span className='text-orange mb-[5px] block font-mono text-[9px] tracking-[.16em]'>
            CORE
          </span>
          <strong className='block text-[17px]'>Rust training core</strong>
          <small className='text-[11px] text-[#738398]'>linfa · ndarray</small>
        </div>
        <div className='relative mx-auto -mb-[2px] w-[80%] rounded-[9px] border border-[#354556] bg-[#151f2d] px-6 py-[18px] shadow-[0_12px_25px_#0003]'>
          <span className='text-orange mb-[5px] block font-mono text-[9px] tracking-[.16em]'>
            EXPORT
          </span>
          <strong className='block text-[17px]'>ONNX artefact</strong>
          <small className='text-[11px] text-[#738398]'>Portable · versioned</small>
        </div>
        <div className='mt-[30px] text-[13px] leading-[1.5] text-[#7e8da0]'>
          Clear boundaries.
          <br />
          <em className='text-orange-2 font-serif text-xl'>Reproducible models.</em>
        </div>
      </div>
      <div>
        <div className={eyebrow}>Built differently, on purpose</div>
        <h2 className={heading}>
          Not a cloud service.
          <br />
          Not a black box.
        </h2>
        <p className='max-w-[480px]'>
          llm.rs is split into a pure-Rust training core, a SQLite metadata layer, a CLI, and a
          desktop UI. Each layer can be understood, tested, and replaced.
        </p>
        <ul className='mt-[35px] grid list-none gap-[21px] p-0'>
          <li className='flex items-start gap-[14px]'>
            <span className='grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]'>
              ✓
            </span>
            <div>
              <strong className='block text-[13px]'>Rust-first foundation</strong>
              <small className='mt-1 block text-[11px] text-[#7f8da0]'>
                Training and export live in crates shared by the desktop app and CLI.
              </small>
            </div>
          </li>
          <li className='flex items-start gap-[14px]'>
            <span className='grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]'>
              ✓
            </span>
            <div>
              <strong className='block text-[13px]'>Reproducible training</strong>
              <small className='mt-1 block text-[11px] text-[#7f8da0]'>
                Every job records the dataset hash, mapping, algorithm, hyperparameters, and seed.
              </small>
            </div>
          </li>
          <li className='flex items-start gap-[14px]'>
            <span className='grid h-[21px] w-[21px] place-items-center rounded-full bg-[#214e3d] text-[11px] text-[#81dcae]'>
              ✓
            </span>
            <div>
              <strong className='block text-[13px]'>Portable by default</strong>
              <small className='mt-1 block text-[11px] text-[#7f8da0]'>
                ONNX is the required artefact; every successful job produces one.
              </small>
            </div>
          </li>
        </ul>
      </div>
    </section>
  );
}
