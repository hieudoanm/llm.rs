import { repositoryUrl, siteName } from '../site';

describe('site lib', () => {
  it('exports siteName', () => {
    expect(typeof siteName).toBe('string');
  });

  it('exports repositoryUrl', () => {
    expect(repositoryUrl).toContain('github.com');
  });
});
