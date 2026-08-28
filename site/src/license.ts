export const PRODUCT_SLUG = 'restore-rehearsal-card';
export const LICENSE_KEY = `sb_license:${PRODUCT_SLUG}`;
export const VERDICT_KEY = `sb_license_verdict:${PRODUCT_SLUG}`;
export const DAY_MS = 86_400_000;

export type LicenseVerdict = {
  token: string;
  valid: boolean;
  checkedAt: number;
  reason: string;
  expiresAt: string | null;
};

type HistoryLike = { replaceState(data: unknown, unused: string, url?: string | URL | null): void };

export function captureReturnLicense(
  location: URL,
  storage: Storage,
  history: HistoryLike,
): string | null {
  const token = location.searchParams.get('license')?.trim();
  if (!token) return storage.getItem(LICENSE_KEY);
  storage.setItem(LICENSE_KEY, token);
  location.searchParams.delete('license');
  history.replaceState({}, '', `${location.pathname}${location.search}${location.hash}`);
  return token;
}

export function readVerdict(storage: Storage, token: string): LicenseVerdict | null {
  try {
    const parsed = JSON.parse(storage.getItem(VERDICT_KEY) ?? '') as LicenseVerdict;
    return parsed.token === token && typeof parsed.checkedAt === 'number' ? parsed : null;
  } catch {
    return null;
  }
}

export function isFresh(verdict: LicenseVerdict, now = Date.now()): boolean {
  return now - verdict.checkedAt < DAY_MS;
}

export async function requestVerdict(
  token: string,
  fetcher: typeof fetch = fetch,
  baseUrl = import.meta.env.VITE_BILLING_BASE_URL || 'https://api.sociobot.in',
): Promise<Omit<LicenseVerdict, 'token' | 'checkedAt'>> {
  const url = `${baseUrl}/api/v1/products/${PRODUCT_SLUG}/verify?license=${encodeURIComponent(token)}`;
  const response = await fetcher(url, { method: 'GET', headers: { Accept: 'application/json' } });
  if (!response.ok) throw new Error(`Verification returned ${response.status}`);
  const result = (await response.json()) as { valid?: unknown; reason?: unknown; expires_at?: unknown };
  if (typeof result.valid !== 'boolean' || typeof result.reason !== 'string') {
    throw new Error('Verification returned an unexpected response');
  }
  return {
    valid: result.valid,
    reason: result.reason,
    expiresAt: typeof result.expires_at === 'string' ? result.expires_at : null,
  };
}
