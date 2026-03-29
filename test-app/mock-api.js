const express = require('express');
const app = express();
app.use(express.json());

const PORT = process.env.PORT || 24042;
const TOKEN = 'mock-test-token-12345';

// --- Auth ---
app.post('/api/v1/users/login', (req, res) => {
  const { username, password } = req.body;
  if (username === 'admin' && password === 'admin') {
    return res.json({
      success: true,
      token: TOKEN,
      user_id: '00000000-0000-0000-0000-000000000001',
      username: 'admin',
      role: 'admin',
      message: 'Login successful',
    });
  }
  res.json({ success: false, token: null, user_id: null, username: null, role: null, message: 'Invalid credentials' });
});

// --- Auth middleware ---
function auth(req, res, next) {
  const header = req.headers.authorization || '';
  if (header === `Bearer ${TOKEN}`) return next();
  res.status(401).json({ error: 'Unauthorized' });
}

// --- Users ---
const users = [
  { id: '00000000-0000-0000-0000-000000000001', username: 'admin', email: 'admin@test.com', role: 'admin', created_at: '2026-01-01 00:00:00', updated_at: null },
];

app.get('/api/v1/users/admin', auth, (req, res) => res.json(users));
app.get('/api/v1/users/:id', auth, (req, res) => {
  const u = users.find(u => u.id === req.params.id);
  u ? res.json(u) : res.status(404).json({ error: 'Not found' });
});

// --- Proxies ---
let proxies = [];
let proxyDomains = [];
let gwnodes = [];
let gateways = [];

app.get('/api/v1/settings/proxies', auth, (req, res) => {
  res.json(proxies.map(p => ({
    proxy: p,
    domains: proxyDomains.filter(d => d.proxy_id === p.id).map(d => ({ id: d.id, sni: d.sni, tls: d.tls })),
  })));
});

app.get('/api/v1/settings/proxy/:id', auth, (req, res) => {
  const p = proxies.find(p => p.id === req.params.id);
  if (!p) return res.status(404).json({ error: 'Not found' });
  res.json({ proxy: p, domains: proxyDomains.filter(d => d.proxy_id === p.id) });
});

app.post('/api/v1/settings/proxy', auth, (req, res) => {
  const { proxy, domains } = req.body;
  proxy.id = proxy.id || crypto.randomUUID();
  proxy.addr_target = proxy.addr_target || '';
  proxies.push(proxy);
  const savedDomains = (domains || []).map(d => {
    d.id = d.id || crypto.randomUUID();
    d.proxy_id = proxy.id;
    proxyDomains.push(d);
    return d;
  });
  res.json({ proxy, domains: savedDomains });
});

app.delete('/api/v1/settings/proxy/:id', auth, (req, res) => {
  const p = proxies.find(p => p.id === req.params.id);
  if (!p) return res.status(404).send('Proxy not found');
  const domCount = proxyDomains.filter(d => d.proxy_id === p.id).length;
  proxies = proxies.filter(p => p.id !== req.params.id);
  proxyDomains = proxyDomains.filter(d => d.proxy_id !== req.params.id);
  res.send(`Proxy '${p.title}' deleted. ${domCount} proxy domains were removed.`);
});

// --- Gateway Nodes ---
app.get('/api/v1/settings/gwnode/list', auth, (req, res) => res.json(gwnodes));
app.get('/api/v1/settings/gwnode/list/:proxy_id', auth, (req, res) => {
  res.json(gwnodes.filter(n => n.proxy_id === req.params.proxy_id));
});
app.get('/api/v1/settings/gwnode/:id', auth, (req, res) => {
  const n = gwnodes.find(n => n.id === req.params.id);
  n ? res.json(n) : res.status(404).json({ error: 'Not found' });
});
app.post('/api/v1/settings/gwnode/set', auth, (req, res) => {
  const node = req.body;
  node.id = node.id || crypto.randomUUID();
  node.priority = node.priority || 100;
  node.domain_id = node.domain_id || null;
  node.domain_name = node.domain_name || null;
  gwnodes.push(node);
  res.json(node);
});
app.post('/api/v1/settings/gwnode/delete', auth, (req, res) => {
  const { id } = req.body;
  const n = gwnodes.find(n => n.id === id);
  if (!n) return res.status(404).json({ message: 'Not found' });
  const gwCount = gateways.filter(g => g.gwnode_id === id).length;
  gwnodes = gwnodes.filter(n => n.id !== id);
  gateways = gateways.filter(g => g.gwnode_id !== id);
  res.json({ message: `Gateway node '${n.title}' deleted successfully along with ${gwCount} associated gateways` });
});

// --- Gateways ---
app.get('/api/v1/settings/gateway/list', auth, (req, res) => res.json(gateways));
app.get('/api/v1/settings/gateway/list/:gwnode_id', auth, (req, res) => {
  res.json(gateways.filter(g => g.gwnode_id === req.params.gwnode_id));
});
app.get('/api/v1/settings/gateway/:id', auth, (req, res) => {
  const g = gateways.find(g => g.id === req.params.id);
  g ? res.json(g) : res.status(404).json({ error: 'Not found' });
});
app.post('/api/v1/settings/gateway/set', auth, (req, res) => {
  const gw = req.body;
  gw.id = gw.id || crypto.randomUUID();
  gateways.push(gw);
  res.json(gw);
});
app.post('/api/v1/settings/gateway/delete', auth, (req, res) => {
  const { id } = req.body;
  gateways = gateways.filter(g => g.id !== id);
  res.json({ message: 'Gateway deleted successfully' });
});

// --- Proxy Domains ---
app.get('/api/v1/settings/proxydomain/list', auth, (req, res) => res.json(proxyDomains));
app.get('/api/v1/settings/proxydomain/list/:proxy_id', auth, (req, res) => {
  res.json(proxyDomains.filter(d => d.proxy_id === req.params.proxy_id));
});
app.get('/api/v1/settings/proxydomain/:id', auth, (req, res) => {
  const d = proxyDomains.find(d => d.id === req.params.id);
  d ? res.json(d) : res.status(404).json({ error: 'Not found' });
});
app.post('/api/v1/settings/proxydomain/set', auth, (req, res) => {
  const d = req.body;
  d.id = d.id || crypto.randomUUID();
  proxyDomains.push(d);
  res.json(d);
});
app.post('/api/v1/settings/proxydomain/delete', auth, (req, res) => {
  const { id } = req.body;
  proxyDomains = proxyDomains.filter(d => d.id !== id);
  res.json({ message: 'Proxy domain deleted successfully' });
});

// --- Sync ---
app.post('/api/v1/sync/proxy', auth, (req, res) => {
  res.json({ status: 'success', message: 'Proxy data updated successfully' });
});
app.post('/api/v1/sync/gateway', auth, (req, res) => {
  res.json({ status: 'success', message: 'Gateway node data updated successfully' });
});

// --- Certificates ---
app.post('/api/v1/settings/certificates/generate', auth, (req, res) => {
  res.json({ status: 'success', message: 'Certificate generated', domain: req.body.domain, expected_renew: '2026-06-29T00:00:00Z' });
});
app.post('/api/v1/settings/certificates/renew-all', auth, (req, res) => {
  res.json({ status: 'success', message: '0 certificates renewed' });
});
app.get('/api/v1/settings/certificates/due-for-renewal', auth, (req, res) => {
  res.json([]);
});

// --- Auto config ---
app.post('/api/v1/settings/auto-config', auth, (req, res) => {
  res.json({ success: true, created: { proxies: 0, domains: 0, gwnodes: 0, gateways: 0 }, error: null });
});
app.get('/api/v1/settings/auto-config', auth, (req, res) => {
  res.type('text/yaml').send('# empty config\nproxy: []\n');
});

// --- SSE Statistics Stream ---
let sseClients = [];
let statsHistory = [];

function generateStats() {
  const now = new Date().toISOString();
  const rand = (min, max) => Math.floor(Math.random() * (max - min + 1)) + min;
  return {
    ts: now,
    gateway: {
      req: rand(50, 200),
      res: rand(48, 198),
      bytes_in: rand(10000, 500000),
      bytes_out: rand(50000, 2000000),
      status: { '200': rand(40, 180), '404': rand(0, 10), '500': rand(0, 3) },
      failed: rand(0, 5),
      bytes_in_min: rand(100, 1000),
      bytes_in_max: rand(10000, 50000),
      bytes_in_avg: rand(1000, 10000) + 0.5,
      bytes_out_min: rand(500, 5000),
      bytes_out_max: rand(50000, 200000),
      bytes_out_avg: rand(5000, 50000) + 0.5,
      stalled_count: rand(0, 2),
    },
    proxy: {
      req: rand(30, 150),
      res: rand(28, 148),
      bytes_in: rand(5000, 300000),
      bytes_out: rand(20000, 1000000),
      status: { '200': rand(25, 140), '301': rand(0, 15), '404': rand(0, 5), '500': rand(0, 2) },
      failed: rand(0, 3),
      bytes_in_min: rand(50, 500),
      bytes_in_max: rand(5000, 30000),
      bytes_in_avg: rand(500, 5000) + 0.5,
      bytes_out_min: rand(200, 2000),
      bytes_out_max: rand(20000, 100000),
      bytes_out_avg: rand(2000, 20000) + 0.5,
      stalled_count: rand(0, 1),
    },
  };
}

// Pre-populate history
for (let i = 0; i < 30; i++) {
  statsHistory.push(generateStats());
}

// Broadcast stats every 5 seconds (faster than real 15s for testing)
setInterval(() => {
  const stats = generateStats();
  statsHistory.push(stats);
  if (statsHistory.length > 120) statsHistory.shift();

  const data = `data: ${JSON.stringify(stats)}\r\n\r\n`;
  sseClients = sseClients.filter(res => {
    try {
      res.write(data);
      return true;
    } catch {
      return false;
    }
  });
}, 5000);

// Ping every 3 seconds
setInterval(() => {
  sseClients = sseClients.filter(res => {
    try {
      res.write(': ping\r\n\r\n');
      return true;
    } catch {
      return false;
    }
  });
}, 3000);

app.get('/api/v1/statistics/stream', auth, (req, res) => {
  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-cache',
    'Connection': 'keep-alive',
  });

  // Send history
  const historyData = `data: ${JSON.stringify(statsHistory)}\r\n\r\n`;
  res.write(historyData);

  sseClients.push(res);
  req.on('close', () => {
    sseClients = sseClients.filter(c => c !== res);
  });
});

app.listen(PORT, () => {
  console.log(`Mock API running on http://localhost:${PORT}`);
  console.log('Login: admin / admin');
  console.log('SSE stats broadcast every 5s (30 history points pre-loaded)');
});
