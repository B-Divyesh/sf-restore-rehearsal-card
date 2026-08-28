import './style.css';
import { captureReturnLicense, isFresh, LICENSE_KEY, LicenseVerdict, readVerdict, requestVerdict, VERDICT_KEY } from './license';

const one = <T extends Element>(selector: string): T | null => document.querySelector<T>(selector);

function setupNetworkNotice(): void {
  const notice = one<HTMLElement>('#offline-notice');
  if (!notice) return;
  const sync = () => { notice.hidden = navigator.onLine; };
  window.addEventListener('online', sync);
  window.addEventListener('offline', sync);
  sync();
}

function setupCopy(): void {
  document.querySelectorAll<HTMLButtonElement>('[data-copy-target]').forEach((button) => {
    button.addEventListener('click', async () => {
      const target = document.getElementById(button.dataset.copyTarget ?? '');
      const status = button.parentElement?.querySelector<HTMLElement>('.copy-status');
      try {
        await navigator.clipboard.writeText(target?.textContent?.trim() ?? '');
        button.querySelector('span')!.textContent = 'Copied';
        if (status) status.textContent = 'Commands copied to your clipboard.';
      } catch {
        if (status) status.textContent = 'Copy was blocked. Select the commands above and copy them manually.';
      }
    });
  });
}

function setupDemo(): void {
  const toggle = one<HTMLButtonElement>('#demo-toggle');
  const lines = [...document.querySelectorAll<HTMLElement>('.demo-line')];
  if (!toggle || lines.length === 0) return;
  let timer: number | undefined;
  let position = 1;
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const stop = () => {
    if (timer) window.clearInterval(timer);
    timer = undefined;
    toggle.textContent = position >= lines.length ? 'Replay demo' : 'Resume demo';
  };
  toggle.addEventListener('click', () => {
    if (timer) { stop(); return; }
    if (position >= lines.length) {
      lines.forEach((line, index) => line.classList.toggle('is-visible', index === 0));
      position = 1;
    }
    toggle.textContent = 'Pause demo';
    if (reduced) {
      lines.forEach((line) => line.classList.add('is-visible'));
      position = lines.length;
      stop();
      return;
    }
    timer = window.setInterval(() => {
      lines[position]?.classList.add('is-visible');
      position += 1;
      if (position >= lines.length) stop();
    }, 520);
  });
}

const kitContents: Record<string, { name: string; body: string }> = {
  runbook: {
    name: 'restore-rehearsal-runbook.md',
    body: `# Weekly restore rehearsal\n\n## Before\n- [ ] Name the disposable target and RTO\n- [ ] Confirm non-production credentials and Docker context\n- [ ] Name the backup artifact and owner\n\n## Run\n- [ ] Execute rrc check\n- [ ] Execute rrc run with exact target confirmation\n- [ ] Verify the signed card\n\n## Review\n- [ ] Did the observed recovery meet RTO?\n- [ ] Which step consumed the most time?\n- [ ] Record one owner and date for every follow-up\n`,
  },
  'decision-log': {
    name: 'rto-decision-log.md',
    body: `# RTO decision log\n\n| Date | Service | Objective | Observed | Result | Decision | Owner | Due |\n| --- | --- | ---: | ---: | --- | --- | --- | --- |\n| YYYY-MM-DD | | | | | | | |\n\nAttach the independently verified Restore Rehearsal Card; do not paste restored data here.\n`,
  },
  'review-pack': {
    name: 'restore-review-and-service-profiles.md',
    body: `# Restore review agenda\n\n1. Verify the card signature.\n2. Compare observed recovery with the declared RTO.\n3. Review the slowest step and every failed assertion.\n4. Assign one owner and due date per action.\n5. Schedule the next rehearsal.\n\n# Service profile worksheets\n\nComplete the same fields for each profile: backup artifact and owner; disposable target; restore argv; health signal; row-count or exit-code assertion; RTO; credentials source; cleanup owner.\n\n## 1. PostgreSQL database\n\n## 2. MySQL database\n\n## 3. SQLite application\n\n## 4. Self-hosted monitoring service\n\n## 5. Object metadata service\n\n## 6. Multi-service Compose stack\n\nNever paste credentials or restored records into this document.\n`,
  },
};

function setupDownloads(): void {
  document.querySelectorAll<HTMLButtonElement>('[data-kit]').forEach((button) => {
    button.addEventListener('click', () => {
      const kit = kitContents[button.dataset.kit ?? ''];
      if (!kit) return;
      const url = URL.createObjectURL(new Blob([kit.body], { type: 'text/markdown' }));
      const anchor = document.createElement('a');
      anchor.href = url;
      anchor.download = kit.name;
      anchor.click();
      URL.revokeObjectURL(url);
    });
  });
}

function setupLicense(): void {
  const form = one<HTMLFormElement>('#license-form');
  const input = one<HTMLInputElement>('#license-token');
  const status = one<HTMLElement>('#license-status');
  const downloads = one<HTMLElement>('#operator-downloads');
  if (!form || !input || !status || !downloads) return;

  const paint = (valid: boolean, message: string): void => {
    downloads.hidden = !valid;
    status.textContent = message;
    status.dataset.state = valid ? 'valid' : 'quiet';
  };
  const verify = async (token: string, allowCached = true): Promise<void> => {
    const cached = readVerdict(localStorage, token);
    if (cached?.valid) paint(true, 'License active. The Operator Pack is available below.');
    if (allowCached && cached && isFresh(cached)) {
      if (!cached.valid) paint(false, 'License no longer active. You can purchase a new license above.');
      return;
    }
    status.textContent = cached?.valid ? 'Operator Pack unlocked. Rechecking license…' : 'Checking license…';
    try {
      const result = await requestVerdict(token);
      const verdict: LicenseVerdict = { token, checkedAt: Date.now(), ...result };
      localStorage.setItem(VERDICT_KEY, JSON.stringify(verdict));
      if (verdict.valid) paint(true, 'License verified. The Operator Pack is available below.');
      else paint(false, 'License no longer active. Check the token or purchase a new license above.');
    } catch {
      if (cached?.valid) paint(true, 'Offline verification deferred. Using your last valid license for now.');
      else paint(false, 'License could not be verified. Check your connection; the free CLI remains available.');
    }
  };

  const token = captureReturnLicense(new URL(window.location.href), localStorage, history);
  if (token) void verify(token);
  form.addEventListener('submit', (event) => {
    event.preventDefault();
    const submitted = input.value.trim();
    if (!submitted) {
      input.setAttribute('aria-invalid', 'true');
      paint(false, 'Enter the complete license token from your receipt.');
      input.focus();
      return;
    }
    input.removeAttribute('aria-invalid');
    localStorage.setItem(LICENSE_KEY, submitted);
    void verify(submitted, false);
  });
}

setupNetworkNotice();
setupCopy();
setupDemo();
setupDownloads();
setupLicense();
