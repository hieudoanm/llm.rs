import type { NextConfig } from 'next';

const NODE_ENV = process.env.NODE_ENV ?? 'development';

const nextConfig: NextConfig = {
  output: 'export',
  basePath: NODE_ENV === 'production' ? '/llm.rs' : '',
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
};

export default nextConfig;
