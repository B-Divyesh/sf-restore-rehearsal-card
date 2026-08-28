import { describe, expect, it, vi } from 'vitest';
import { captureReturnLicense, DAY_MS, isFresh, LICENSE_KEY, readVerdict, requestVerdict, VERDICT_KEY } from './license';

describe('license handling', () => {
  it('stores return token and strips it from visible URL', () => {
    const replaceState = vi.fn();
    const token = captureReturnLicense(new URL('https://example.test/?ref=docs&license=abc123#pack'), localStorage, { replaceState });
    expect(token).toBe('abc123');
    expect(localStorage.getItem(LICENSE_KEY)).toBe('abc123');
    expect(replaceState).toHaveBeenCalledWith({}, '', '/?ref=docs#pack');
  });

  it('uses only a verdict for the same token', () => {
    localStorage.setItem(VERDICT_KEY, JSON.stringify({ token: 'a', valid: true, checkedAt: 100, reason: 'ok', expiresAt: null }));
    expect(readVerdict(localStorage, 'a')?.valid).toBe(true);
    expect(readVerdict(localStorage, 'b')).toBeNull();
  });

  it('limits successful verification to once per day', () => {
    const verdict = { token: 'a', valid: true, checkedAt: 1_000, reason: 'ok', expiresAt: null };
    expect(isFresh(verdict, 1_000 + DAY_MS - 1)).toBe(true);
    expect(isFresh(verdict, 1_000 + DAY_MS)).toBe(false);
  });

  it('encodes tokens sent to the product-specific endpoint', async () => {
    let requested = '';
    const fetcher = vi.fn(async (input: string | URL | Request) => {
      requested = String(input);
      return new Response(JSON.stringify({ valid: true, reason: 'ok', expires_at: null }), { status: 200 });
    });
    await requestVerdict('a+b&c', fetcher as typeof fetch, 'https://billing.test');
    expect(requested).toBe('https://billing.test/api/v1/products/restore-rehearsal-card/verify?license=a%2Bb%26c');
  });

  it('@claim:production-license-verification defaults to the production Sociobot API', async () => {
    let requested = '';
    const fetcher = vi.fn(async (input: string | URL | Request) => {
      requested = String(input);
      return new Response(JSON.stringify({ valid: true, reason: 'ok', expires_at: null }), { status: 200 });
    });
    await requestVerdict('production-token', fetcher as typeof fetch);
    expect(requested).toBe('https://api.sociobot.in/api/v1/products/restore-rehearsal-card/verify?license=production-token');
  });
});
