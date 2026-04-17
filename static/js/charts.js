const sparklineInstances = {};

function initSparkline(canvasId, checks) {
  const canvas = document.getElementById(canvasId);
  if (!canvas) return;

  if (sparklineInstances[canvasId]) {
    sparklineInstances[canvasId].destroy();
  }

  const points = checks
    .filter(c => c.response_time_ms != null)
    .slice(0, 30)
    .reverse();

  if (points.length === 0) return;

  const labels = points.map(() => '');
  const data = points.map(c => c.response_time_ms);
  const colors = points.map(c => c.status === 'up' ? '#a6e3a1' : '#f38ba8');

  sparklineInstances[canvasId] = new Chart(canvas, {
    type: 'line',
    data: {
      labels,
      datasets: [{
        data,
        borderColor: '#89b4fa',
        borderWidth: 1.5,
        pointBackgroundColor: colors,
        pointRadius: 3,
        pointHoverRadius: 4,
        fill: true,
        backgroundColor: 'rgba(137, 180, 250, 0.08)',
        tension: 0.3,
      }],
    },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      animation: false,
      plugins: { legend: { display: false }, tooltip: {
        callbacks: {
          label: ctx => `${ctx.parsed.y}ms`,
        },
        backgroundColor: '#1e1e2e',
        bodyColor: '#cdd6f4',
        borderColor: '#2d2d3d',
        borderWidth: 1,
      }},
      scales: {
        x: { display: false },
        y: {
          display: false,
          beginAtZero: true,
        },
      },
    },
  });
}
