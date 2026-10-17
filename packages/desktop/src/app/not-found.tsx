import Link from 'next/link';

export const NotFoundRoute = () => (
  <main className='mx-auto max-w-5xl px-4 py-20 text-center'>
    <h1 className='text-text-primary text-4xl font-bold'>404</h1>
    <p className='text-text-secondary mt-4 text-lg'>Page not found.</p>
    <Link href='/jobs/new' className='btn btn-primary mt-8 inline-block'>
      New job
    </Link>
  </main>
);

export default NotFoundRoute;
