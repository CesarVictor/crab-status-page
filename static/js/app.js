const API = {
  stats:         () => fetch('/api/stats').then(r => r.json()),
  services:      () => fetch('/api/services').then(r => r.json()),
  checks:     (id) => fetch(`/api/services/${id}/checks?limit=30`).then(r => r.json()),
  uptime:     (id) => fetch(`/api/services/${id}/uptime`).then(r => r.json()),
  incidents:     () => fetch('/api/incidents/active').then(r => r.json()),
  createService: (d) => fetch('/api/services', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(d),
  }),
  deleteService: (id) => fetch(`/api/services/${id}`, { method: 'DELETE' }),
};

let refreshTimer;
let countdown = 30;

async function loadDashboard() {
  try {
    const [stats, services, incidents] = await Promise.all([
      API.stats(),
      API.services(),
      API.incidents(),
    ]);

    renderStats(stats);
    renderBanner(incidents, services);
    renderIncidents(incidents);

    const details = await Promise.all(
      services.map(async (svc) => {
        const [checks, uptime] = await Promise.all([
          API.checks(svc.id),
          API.uptime(svc.id),
        ]);
        return { svc, checks, uptime };
      })
    );

    renderGrid(details);
  } catch (err) {
    console.error('Dashboard load failed:', err);
  }
}

function renderStats(stats) {
  document.getElementById('stat-services').textContent = stats.active_services ?? '—';
  document.getElementById('stat-uptime').textContent =
    stats.avg_uptime_24h != null ? `${stats.avg_uptime_24h.toFixed(1)}%` : '—';
  document.getElementById('stat-checks').textContent = stats.total_checks_today ?? '—';
}

function renderBanner(incidents, services) {
  const banner = document.getElementById('status-banner');
  if (incidents.length === 0) {
    banner.className = 'status-banner ok';
    banner.textContent = `✓ All ${services.length} service${services.length !== 1 ? 's' : ''} operational`;
  } else {
    banner.className = 'status-banner incident';
    banner.textContent = `⚠ ${incidents.length} active incident${incidents.length !== 1 ? 's' : ''}`;
  }
}

function renderIncidents(incidents) {
  const section = document.getElementById('incidents-section');
  const list = document.getElementById('incidents-list');

  if (incidents.length === 0) {
    section.style.display = 'none';
    return;
  }

  section.style.display = '';
  list.innerHTML = incidents.map(inc => `
    <div class="incident-item">
      <div>
        <div class="incident-service">${escHtml(inc.service_id)}</div>
        ${inc.cause ? `<div class="incident-cause">${escHtml(inc.cause)}</div>` : ''}
      </div>
      <div class="incident-time">${relativeTime(inc.started_at)}</div>
    </div>
  `).join('');
}

function renderGrid(details) {
  const grid = document.getElementById('services-grid');

  if (details.length === 0) {
    grid.innerHTML = `
      <div class="empty-state">
        <img src="/img/ferris-checking.svg" style="width:80px;height:80px;opacity:0.5">
        <p>No services yet. Add one above.</p>
      </div>`;
    return;
  }

  grid.innerHTML = details.map(({ svc, checks, uptime }) =>
    buildCard(svc, checks, uptime)
  ).join('');

  details.forEach(({ svc, checks }) => {
    initSparkline(`sparkline-${svc.id}`, checks);
  });
}

function buildCard(svc, checks, uptime) {
  const latest = checks[0];
  const status = latest ? latest.status : 'unknown';
  const isDown = status === 'down';

  const responseTime = latest?.response_time_ms;
  const rtClass = responseTime == null ? '' :
    responseTime < 300  ? 'fast' :
    responseTime < 1000 ? 'slow' : 'very-slow';

  const dots = buildDots(checks);

  return `
    <div class="service-card ${isDown ? 'is-down' : ''}" id="card-${svc.id}">
      <div class="service-card-header">
        <div class="service-card-meta">
          <div class="service-name">${escHtml(svc.name)}</div>
          <div class="service-url">${escHtml(svc.url)}</div>
        </div>
        <img
          src="${ferrisImg(status)}"
          alt="${status}"
          class="service-ferris ${ferrisClass(status)}"
        >
      </div>

      <div class="service-status-row">
        <span class="status-badge ${status}">${status.toUpperCase()}</span>
        ${responseTime != null
          ? `<span class="response-time ${rtClass}">${responseTime}ms</span>`
          : '<span class="response-time">—</span>'
        }
      </div>

      <div class="uptime-dots" title="Last ${checks.length} checks">${dots}</div>

      <div class="uptime-stats">
        <span class="uptime-stat">24h <strong>${uptime.uptime_24h.toFixed(1)}%</strong></span>
        <span class="uptime-stat">·</span>
        <span class="uptime-stat">7d <strong>${uptime.uptime_7d.toFixed(1)}%</strong></span>
        <span class="uptime-stat">·</span>
        <span class="uptime-stat">30d <strong>${uptime.uptime_30d.toFixed(1)}%</strong></span>
      </div>

      <div class="sparkline-wrap">
        <canvas id="sparkline-${svc.id}"></canvas>
      </div>

      <div class="service-card-footer">
        <span class="last-checked">
          ${latest ? `checked ${relativeTime(latest.checked_at)}` : 'never checked'}
        </span>
        <button class="btn btn-danger" onclick="deleteService('${svc.id}')" title="Remove service">✕</button>
      </div>
    </div>
  `;
}

function buildDots(checks) {
  const slots = 30;
  const filled = checks.slice(0, slots);
  const dots = [];

  for (let i = 0; i < slots; i++) {
    const check = filled[slots - 1 - i];
    const cls = check ? check.status : 'unknown';
    const label = check
      ? `${check.status.toUpperCase()} — ${relativeTime(check.checked_at)}`
      : 'No data';
    dots.push(`<div class="uptime-dot ${cls}" title="${label}"></div>`);
  }

  return dots.join('');
}

async function deleteService(id) {
  if (!confirm('Remove this service from monitoring?')) return;
  try {
    await API.deleteService(id);
    document.getElementById(`card-${id}`)?.remove();
    loadDashboard();
  } catch (err) {
    console.error('Delete failed:', err);
  }
}

// ── Add service form ──

document.getElementById('toggle-form-btn').addEventListener('click', () => {
  const form = document.getElementById('add-form');
  const btn  = document.getElementById('toggle-form-btn');
  const open = form.style.display === 'none';
  form.style.display = open ? '' : 'none';
  btn.textContent = open ? '✕ Cancel' : '+ Add service';
});

document.getElementById('add-form').addEventListener('submit', async (e) => {
  e.preventDefault();
  const errEl = document.getElementById('form-error');
  errEl.style.display = 'none';

  const payload = {
    name: document.getElementById('form-name').value.trim(),
    url:  document.getElementById('form-url').value.trim(),
    check_interval_seconds: parseInt(document.getElementById('form-interval').value, 10),
    expected_status_code:   parseInt(document.getElementById('form-status').value, 10),
  };

  try {
    const res = await API.createService(payload);
    if (!res.ok) {
      const body = await res.json();
      errEl.textContent = body.error ?? 'Failed to add service';
      errEl.style.display = '';
      return;
    }
    document.getElementById('add-form').reset();
    document.getElementById('add-form').style.display = 'none';
    document.getElementById('toggle-form-btn').textContent = '+ Add service';
    loadDashboard();
  } catch (err) {
    errEl.textContent = 'Network error';
    errEl.style.display = '';
  }
});

// ── Manual refresh ──
document.getElementById('refresh-btn').addEventListener('click', () => {
  resetCountdown();
  loadDashboard();
});

// ── Auto-refresh countdown ──
function resetCountdown() {
  countdown = 30;
  clearInterval(refreshTimer);
  refreshTimer = setInterval(() => {
    countdown--;
    document.getElementById('refresh-countdown').textContent = countdown;
    if (countdown <= 0) {
      resetCountdown();
      loadDashboard();
    }
  }, 1000);
}

// ── Helpers ──

function escHtml(str) {
  return String(str)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function relativeTime(iso) {
  const diff = Math.floor((Date.now() - new Date(iso).getTime()) / 1000);
  if (diff < 5)   return 'just now';
  if (diff < 60)  return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

// ── Boot ──
loadDashboard();
resetCountdown();
