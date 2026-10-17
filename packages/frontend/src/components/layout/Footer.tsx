export const Footer = () => {
  // An empty value means "use the default", so the fallback is a `||` and not a
  // `??` — the compose file passes an empty string when the variable is unset.
  const configuredDocsUrl = process.env.NEXT_PUBLIC_API_DOCS_URL?.trim();
  const apiDocsUrl =
    configuredDocsUrl ||
    `${(process.env.NEXT_PUBLIC_API_URL ?? 'http://127.0.0.1:8000').replace(/\/$/, '')}/docs`;

  return (
    <footer className='border-border border-t'>
      <div className='mx-auto flex max-w-5xl flex-col items-center justify-between gap-4 px-4 py-8 sm:flex-row'>
        <span className='text-text-tertiary text-xs'>scikit-learn-client — local studio.</span>
        <div className='flex items-center gap-4'>
          <a
            href={apiDocsUrl}
            target='_blank'
            rel='noreferrer'
            className='link text-text-tertiary text-xs'>
            API documentation
          </a>
          <p className='text-text-tertiary text-xs'>Not a production AutoML service.</p>
        </div>
      </div>
    </footer>
  );
};
