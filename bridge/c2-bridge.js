const { WebSocketServer } = require("ws");

const PORT = process.env.C2_PORT || 9100;
const wss = new WebSocketServer({ port: PORT, path: "/c2" });

const contacts = [
  { id: "T-001", lat: 33.42, lon: -112.08, alt_ft: 25000, heading: 90, speed_kts: 480, contact_type: "air", threat: "hostile", label: "BANDIT-1", source: "radar" },
  { id: "T-002", lat: 33.80, lon: -111.50, alt_ft: 500, heading: 270, speed_kts: 560, contact_type: "air", threat: "hostile", label: "BANDIT-2", source: "radar" },
  { id: "F-101", lat: 33.30, lon: -112.50, alt_ft: 22000, heading: 45, speed_kts: 420, contact_type: "air", threat: "friendly", label: "VIPER-1", source: "datalink" },
  { id: "F-102", lat: 33.55, lon: -112.20, alt_ft: 18000, heading: 180, speed_kts: 380, contact_type: "air", threat: "friendly", label: "VIPER-2", source: "datalink" },
  { id: "D-201", lat: 33.60, lon: -111.90, alt_ft: 15000, heading: 120, speed_kts: 165, contact_type: "drone", threat: "friendly", label: "REAPER-1", source: "gcs" },
  { id: "S-301", lat: 33.10, lon: -112.30, alt_ft: 0, heading: 0, speed_kts: 0, contact_type: "ground", threat: "neutral", label: "CP-ALPHA", source: "intel" },
  { id: "M-401", lat: 33.70, lon: -111.70, alt_ft: 100, heading: 225, speed_kts: 620, contact_type: "missile", threat: "hostile", label: "SAM-TRACK", source: "ew" },
  { id: "V-501", lat: 33.25, lon: -112.60, alt_ft: 0, heading: 15, speed_kts: 35, contact_type: "surface", threat: "unknown", label: "CONVOY-1", source: "isr" },
];

function tick() {
  const t = Date.now() / 1000;
  for (const c of contacts) {
    c.lat += Math.sin(t * 0.1 + c.id.charCodeAt(2)) * 0.0003;
    c.lon += Math.cos(t * 0.08 + c.id.charCodeAt(2)) * 0.0004;
    c.heading = (c.heading + Math.sin(t * 0.05) * 0.5 + 360) % 360;
    if (c.speed_kts > 0) c.speed_kts += Math.sin(t * 0.2) * 2;
  }
}

function broadcast() {
  tick();
  const msg = JSON.stringify({ type: "ContactUpdate", data: contacts });
  for (const ws of wss.clients) {
    if (ws.readyState === 1) ws.send(msg);
  }
}

setInterval(broadcast, 1000);

wss.on("connection", (ws) => {
  console.log(`[C2] client connected (${wss.clients.size} total)`);
  ws.send(JSON.stringify({ type: "ContactUpdate", data: contacts }));
  ws.on("close", () => console.log(`[C2] client disconnected (${wss.clients.size} total)`));
});

console.log(`[C2 BRIDGE] ws://127.0.0.1:${PORT}/c2 — ${contacts.length} contacts`);
