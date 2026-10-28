export class BabylonCore {
  constructor(opts = {}) {
    this.components = new Map();
    this.feeds = new Map();
    this.classifiers = [];
    this.interval = opts.pollInterval || 3000;
    this.running = false;
    this.onData = opts.onData || null;
  }

  register(name, component) {
    this.components.set(name, component);
    return this;
  }

  addFeed(name, feedFn) {
    this.feeds.set(name, feedFn);
    return this;
  }

  addClassifier(fn) {
    this.classifiers.push(fn);
    return this;
  }

  async classify(devices) {
    let result = devices;
    for (const fn of this.classifiers) result = await fn(result);
    return result;
  }

  async poll() {
    const raw = [];
    for (const [name, fn] of this.feeds) {
      try { raw.push(...await fn()); } catch (e) { console.warn(`feed:${name}`, e); }
    }
    const classified = await this.classify(raw);
    if (this.onData) this.onData(classified);
    for (const [, comp] of this.components) {
      if (comp.feed) comp.feed(classified);
    }
    return classified;
  }

  start() {
    this.running = true;
    const tick = async () => {
      if (!this.running) return;
      await this.poll();
      setTimeout(tick, this.interval);
    };
    tick();
  }

  stop() { this.running = false; }
}

export function vendorClassifier(device) {
  const known = {
    'Apple': 'friendly', 'Samsung': 'neutral', 'Google': 'neutral',
    'Intel': 'friendly', 'Raspberry': 'friendly', 'Amazon': 'neutral',
    'Espressif': 'iot', 'Unknown': 'hostile',
  };
  const vendor = device.vendor || 'Unknown';
  const match = Object.keys(known).find(k => vendor.includes(k));
  return { ...device, threat: match ? known[match] : 'unknown' };
}

export function signalClassifier(device) {
  const rssi = device.rssi || -100;
  let proximity;
  if (rssi > -40) proximity = 'close';
  else if (rssi > -60) proximity = 'near';
  else if (rssi > -75) proximity = 'medium';
  else proximity = 'far';
  return { ...device, proximity };
}

export async function wifiFeed() {
  const resp = await fetch('/api/wifi/scan');
  if (!resp.ok) throw new Error(`scan failed: ${resp.status}`);
  const data = await resp.json();
  return data.devices || data;
}

export function mockWifiFeed() {
  const macs = [
    { mac: 'AA:BB:CC:11:22:33', vendor: 'Apple', name: 'iPhone-James' },
    { mac: 'DD:EE:FF:44:55:66', vendor: 'Samsung', name: 'Galaxy-S24' },
    { mac: '11:22:33:44:55:66', vendor: 'Espressif', name: 'ESP32-Sensor' },
    { mac: '77:88:99:AA:BB:CC', vendor: 'Intel', name: 'Laptop-Work' },
    { mac: 'DE:AD:BE:EF:00:01', vendor: 'Unknown', name: '' },
    { mac: 'CA:FE:BA:BE:00:02', vendor: 'Raspberry', name: 'Pi-Server' },
    { mac: 'FE:ED:FA:CE:00:03', vendor: 'Amazon', name: 'Echo-Dot' },
    { mac: 'BA:DC:0D:E0:00:04', vendor: 'Unknown', name: '' },
  ];
  return macs.map((m, i) => ({
    ...m,
    rssi: -30 - Math.random() * 60 | 0,
    channel: [1, 6, 11, 36, 44][Math.random() * 5 | 0],
    x: Math.cos(Date.now() / 5000 + i * 1.2) * (3 + i * 1.1),
    z: Math.sin(Date.now() / 4000 + i * 0.9) * (3 + i * 0.9),
    lastSeen: Date.now(),
    firstSeen: Date.now() - Math.random() * 300000,
  }));
}
