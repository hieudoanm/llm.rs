import Link from 'next/link';
import { PiCpu, PiArrowsLeftRight, PiDownload } from 'react-icons/pi';

const Home = () => (
  <div className='mx-auto max-w-5xl px-4 pt-16 pb-12'>
    <section className='mx-auto mb-16 max-w-3xl text-center'>
      <PiCpu className='text-primary mx-auto mb-6 text-6xl' aria-hidden='true' />
      <h1 className='text-text-primary mb-4 text-3xl font-bold tracking-tight sm:text-5xl'>
        scikit-learn-client
      </h1>
      <p className='text-text-secondary mb-8 text-base leading-relaxed sm:text-lg'>
        Upload a CSV, choose a model, download ONNX. Train locally in minutes — no infrastructure
        required.
      </p>
      <div className='flex flex-wrap items-center justify-center gap-3'>
        <Link href='/jobs/new' className='btn btn-primary btn-lg w-full sm:w-auto'>
          Train a model
        </Link>
        <Link href='/jobs' className='btn btn-outline btn-lg w-full sm:w-auto'>
          View jobs
        </Link>
      </div>
    </section>

    <section className='border-border bg-background-secondary mb-16 border-t'>
      <div className='mx-auto max-w-5xl px-4 py-16'>
        <div className='mx-auto mb-12 max-w-2xl text-center'>
          <h2 className='text-text-primary mb-3 text-2xl font-semibold tracking-tight sm:text-3xl'>
            Everything you need to train
          </h2>
          <p className='text-text-secondary text-base'>
            A focused set of features that make training effortless.
          </p>
        </div>
        <div className='grid gap-6 sm:grid-cols-3'>
          <div className='card bg-surface border-border rounded-xl border p-6 shadow-sm'>
            <span className='bg-primary-muted text-primary mb-4 flex h-10 w-10 items-center justify-center rounded-full'>
              <PiCpu className='icon-sm' aria-hidden='true' />
            </span>
            <h3 className='text-text-primary mb-2 text-base font-semibold'>CSV upload</h3>
            <p className='text-text-secondary text-sm leading-relaxed'>
              Drag and drop a CSV file. Columns are detected and previewed before you map features
              and labels.
            </p>
          </div>
          <div className='card bg-surface border-border rounded-xl border p-6 shadow-sm'>
            <span className='bg-accent text-accent-content mb-4 flex h-10 w-10 items-center justify-center rounded-full'>
              <PiArrowsLeftRight className='icon-sm' aria-hidden='true' />
            </span>
            <h3 className='text-text-primary mb-2 text-base font-semibold'>Task-aware models</h3>
            <p className='text-text-secondary text-sm leading-relaxed'>
              Classification, regression, semi-supervised self-training, clustering, and anomaly
              detection — decision trees, ensembles, linear models, and more.
            </p>
          </div>
          <div className='card bg-surface border-border rounded-xl border p-6 shadow-sm'>
            <span className='bg-success-bg text-success-fg mb-4 flex h-10 w-10 items-center justify-center rounded-full'>
              <PiDownload className='icon-sm' aria-hidden='true' />
            </span>
            <h3 className='text-text-primary mb-2 text-base font-semibold'>ONNX export</h3>
            <p className='text-text-secondary text-sm leading-relaxed'>
              Download a trained model as ONNX so it runs anywhere — no Python runtime needed at
              inference time.
            </p>
          </div>
        </div>
      </div>
    </section>

    <section className='mb-16'>
      <div className='mx-auto max-w-5xl px-4'>
        <h2 className='text-text-primary mb-10 text-2xl font-semibold tracking-tight sm:text-3xl'>
          How it works
        </h2>
        <div className='grid gap-6 sm:grid-cols-3'>
          <div className='text-center'>
            <div className='bg-primary text-primary-content mx-auto mb-4 flex h-10 w-10 items-center justify-center rounded-full'>
              1
            </div>
            <h3 className='text-text-primary mb-3 text-base font-semibold'>Upload dataset</h3>
            <p className='text-text-secondary text-sm leading-relaxed'>
              Select your CSV and review the detected columns and null counts.
            </p>
          </div>
          <div className='text-center'>
            <div className='bg-primary text-primary-content mx-auto mb-4 flex h-10 w-10 items-center justify-center rounded-full'>
              2
            </div>
            <h3 className='text-text-primary mb-3 text-base font-semibold'>Map and configure</h3>
            <p className='text-text-secondary text-sm leading-relaxed'>
              Pick feature columns, the label column, the task, and the algorithm.
            </p>
          </div>
          <div className='text-center'>
            <div className='bg-primary text-primary-content mx-auto mb-4 flex h-10 w-10 items-center justify-center rounded-full'>
              3
            </div>
            <h3 className='text-text-primary mb-3 text-base font-semibold'>Train and download</h3>
            <p className='text-text-secondary text-sm leading-relaxed'>
              Start training, watch the lifecycle, then download ONNX, pickle, or metrics.
            </p>
          </div>
        </div>
      </div>
    </section>

    <section className='border-border bg-primary border-t'>
      <div className='mx-auto max-w-3xl px-4 py-16 text-center'>
        <h2 className='text-primary-content mb-4 text-2xl font-semibold tracking-tight sm:text-3xl'>
          Ready to train your first model?
        </h2>
        <p className='text-primary-content/80 mx-auto mt-3 mb-8 max-w-xl text-sm leading-relaxed'>
          It takes less than a minute to go from CSV to ONNX.
        </p>
        <Link href='/jobs/new' className='btn btn-lg w-full sm:w-auto'>
          Start training
        </Link>
      </div>
    </section>
  </div>
);

export default Home;
