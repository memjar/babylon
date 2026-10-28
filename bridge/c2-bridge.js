const { WebSocketServer } = require("ws");

const PORT = process.env.C2_PORT || 9100;
const wss = new WebSocketServer({ port: PORT, path: "/c2" });

// tactical contacts (military C2)
const tactical = [
  { id: "T-001", lat: 33.42, lon: -112.08, alt_ft: 25000, heading: 90, speed_kts: 480, contact_type: "air", threat: "hostile", label: "BANDIT-1", source: "radar" },
  { id: "T-002", lat: 33.80, lon: -111.50, alt_ft: 500, heading: 270, speed_kts: 560, contact_type: "air", threat: "hostile", label: "BANDIT-2", source: "radar" },
  { id: "F-101", lat: 33.30, lon: -112.50, alt_ft: 22000, heading: 45, speed_kts: 420, contact_type: "air", threat: "friendly", label: "VIPER-1", source: "datalink" },
  { id: "F-102", lat: 33.55, lon: -112.20, alt_ft: 18000, heading: 180, speed_kts: 380, contact_type: "air", threat: "friendly", label: "VIPER-2", source: "datalink" },
  { id: "D-201", lat: 33.60, lon: -111.90, alt_ft: 15000, heading: 120, speed_kts: 165, contact_type: "drone", threat: "friendly", label: "REAPER-1", source: "gcs" },
  { id: "S-301", lat: 33.10, lon: -112.30, alt_ft: 0, heading: 0, speed_kts: 0, contact_type: "ground", threat: "neutral", label: "CP-ALPHA", source: "intel" },
  { id: "M-401", lat: 33.70, lon: -111.70, alt_ft: 100, heading: 225, speed_kts: 620, contact_type: "missile", threat: "hostile", label: "SAM-TRACK", source: "ew" },
  { id: "V-501", lat: 33.25, lon: -112.60, alt_ft: 0, heading: 15, speed_kts: 35, contact_type: "surface", threat: "unknown", label: "CONVOY-1", source: "isr" },
];

// globe layer data (live feeds)
let flights = [];
let quakes = [];
let fires = [];
let weather = [];
let satellites = [];
let cctv = [];

function randBetween(a, b) { return a + Math.random() * (b - a); }

function demoFlights(n) {
  return Array.from({ length: n }, (_, i) => ({
    id: `FL-${i}`, lat: randBetween(-60, 70), lon: randBetween(-180, 180),
    alt_ft: randBetween(5000, 41000), heading: randBetween(0, 360),
    speed_kts: randBetween(200, 500), contact_type: "air", threat: "neutral",
    label: `FL${String(1000 + i)}`, source: "adsb",
  }));
}

function demoQuakes(n) {
  return Array.from({ length: n }, (_, i) => ({
    id: `EQ-${i}`, lat: randBetween(-60, 60), lon: randBetween(-180, 180),
    alt_ft: 0, heading: 0, speed_kts: 0, contact_type: "ground", threat: "hostile",
    label: `M${(2 + Math.random() * 5).toFixed(1)}`, source: "usgs",
  }));
}

function demoFires(n) {
  return Array.from({ length: n }, (_, i) => ({
    id: `FI-${i}`, lat: randBetween(-50, 65), lon: randBetween(-180, 180),
    alt_ft: 0, heading: 0, speed_kts: 0, contact_type: "ground", threat: "unknown",
    label: `FIRE-${i}`, source: "firms",
  }));
}

function demoWeather(n) {
  return Array.from({ length: n }, (_, i) => ({
    id: `WX-${i}`, lat: randBetween(-40, 50), lon: randBetween(-180, 180),
    alt_ft: randBetween(0, 50000), heading: randBetween(0, 360), speed_kts: randBetween(0, 80),
    contact_type: "air", threat: "friendly", label: `STORM-${i}`, source: "eonet",
  }));
}

async function fetchFlights() {
  try {
    const r = await fetch("https://opensky-network.org/api/states/all", { signal: AbortSignal.timeout(10000) });
    if (!r.ok) throw new Error(r.status);
    const json = await r.json();
    flights = (json.states || []).slice(0, 150).map((s, i) => ({
      id: `FL-${i}`, lat: s[6] || 0, lon: s[5] || 0,
      alt_ft: (s[7] || 0) * 3.281, heading: s[10] || 0,
      speed_kts: (s[9] || 0) * 1.944, contact_type: "air", threat: "neutral",
      label: (s[1] || "").trim() || `ADSB-${i}`, source: "adsb",
    })).filter(f => f.lat && f.lon);
    console.log(`[GLOBE] ${flights.length} flights from OpenSky`);
  } catch {
    if (!flights.length) flights = demoFlights(150);
    console.log("[GLOBE] flights: using demo data");
  }
}

async function fetchQuakes() {
  try {
    const r = await fetch("https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_day.geojson", { signal: AbortSignal.timeout(10000) });
    if (!r.ok) throw new Error(r.status);
    const json = await r.json();
    quakes = (json.features || []).map((f, i) => ({
      id: `EQ-${i}`, lat: f.geometry.coordinates[1], lon: f.geometry.coordinates[0],
      alt_ft: 0, heading: 0, speed_kts: 0, contact_type: "ground", threat: "hostile",
      label: `M${f.properties.mag?.toFixed(1) || "?"} ${(f.properties.place || "").slice(0, 20)}`, source: "usgs",
    }));
    console.log(`[GLOBE] ${quakes.length} quakes from USGS`);
  } catch {
    if (!quakes.length) quakes = demoQuakes(30);
    console.log("[GLOBE] quakes: using demo data");
  }
}

async function fetchWeather() {
  try {
    const r = await fetch("https://eonet.gsfc.nasa.gov/api/v3/events?status=open&limit=30", { signal: AbortSignal.timeout(10000) });
    if (!r.ok) throw new Error(r.status);
    const json = await r.json();
    weather = (json.events || []).map((e, i) => {
      const geo = e.geometry?.[0];
      const coords = geo?.coordinates;
      const lat = Array.isArray(coords?.[0]) ? coords[0][1] : coords?.[1];
      const lon = Array.isArray(coords?.[0]) ? coords[0][0] : coords?.[0];
      return {
        id: `WX-${i}`, lat: lat || 0, lon: lon || 0,
        alt_ft: 0, heading: 0, speed_kts: 0, contact_type: "air", threat: "friendly",
        label: (e.title || "").slice(0, 25), source: "eonet",
      };
    }).filter(w => w.lat && w.lon);
    console.log(`[GLOBE] ${weather.length} weather events from EONET`);
  } catch {
    if (!weather.length) weather = demoWeather(20);
    console.log("[GLOBE] weather: using demo data");
  }
}

async function fetchFires() {
  if (!fires.length) fires = demoFires(60);
  console.log(`[GLOBE] ${fires.length} fires (demo)`);
}

async function fetchSatellites() {
  try {
    const r = await fetch("https://celestrak.org/NORAD/elements/gp.php?GROUP=stations&FORMAT=json", { signal: AbortSignal.timeout(10000) });
    if (!r.ok) throw new Error(r.status);
    const json = await r.json();
    satellites = json.slice(0, 80).map((s, i) => ({
      id: `SAT-${i}`, lat: s.INCLINATION ? randBetween(-s.INCLINATION, s.INCLINATION) : randBetween(-60, 60),
      lon: randBetween(-180, 180),
      alt_ft: (s.SEMIMAJOR_AXIS ? (s.SEMIMAJOR_AXIS - 6371) * 3281 : 250000),
      heading: randBetween(0, 360), speed_kts: 15000,
      contact_type: "air", threat: "neutral",
      label: `SAT-${(s.OBJECT_NAME || "").slice(0, 15)}`, source: "celestrak",
    }));
    console.log(`[GLOBE] ${satellites.length} satellites from CelesTrak`);
  } catch {
    if (!satellites.length) {
      satellites = Array.from({ length: 80 }, (_, i) => ({
        id: `SAT-${i}`, lat: randBetween(-55, 55), lon: randBetween(-180, 180),
        alt_ft: randBetween(200000, 400000), heading: randBetween(0, 360), speed_kts: 15000,
        contact_type: "air", threat: "neutral",
        label: `SAT-NORAD-${1000 + i}`, source: "celestrak",
      }));
    }
    console.log("[GLOBE] satellites: using demo data");
  }
}

function generateCCTV() {
  const cities = [
    [51.51, -0.13, "LONDON"], [40.71, -74.01, "NYC"], [35.68, 139.69, "TOKYO"],
    [48.86, 2.35, "PARIS"], [-33.87, 151.21, "SYDNEY"], [55.76, 37.62, "MOSCOW"],
    [39.91, 116.40, "BEIJING"], [19.43, -99.13, "MEXICO"], [28.61, 77.21, "DELHI"],
    [1.35, 103.82, "SINGAPORE"], [-23.55, -46.63, "SAOPAULO"], [37.57, 126.98, "SEOUL"],
    [52.52, 13.41, "BERLIN"], [41.01, 28.98, "ISTANBUL"], [25.20, 55.27, "DUBAI"],
    [34.05, -118.24, "LA"], [43.65, -79.38, "TORONTO"], [22.32, 114.17, "HONGKONG"],
  ];
  cctv = [];
  for (const [lat, lon, city] of cities) {
    const n = 3 + Math.floor(Math.random() * 8);
    for (let i = 0; i < n; i++) {
      cctv.push({
        id: `CCTV-${city}-${i}`, lat: lat + randBetween(-0.5, 0.5), lon: lon + randBetween(-0.5, 0.5),
        alt_ft: 0, heading: randBetween(0, 360), speed_kts: 0,
        contact_type: "ground", threat: "neutral",
        label: `CAM-${city}-${i}`, source: "cctv",
      });
    }
  }
  console.log(`[GLOBE] ${cctv.length} CCTV nodes across ${cities.length} cities`);
}

async function refreshGlobeData() {
  await Promise.allSettled([fetchFlights(), fetchQuakes(), fetchWeather(), fetchFires(), fetchSatellites()]);
  if (!cctv.length) generateCCTV();
  const total = flights.length + quakes.length + weather.length + fires.length + satellites.length + cctv.length;
  console.log(`[GLOBE] refreshed: ${total} total globe contacts`);
}

function tickTactical() {
  const t = Date.now() / 1000;
  for (const c of tactical) {
    c.lat += Math.sin(t * 0.1 + c.id.charCodeAt(2)) * 0.0003;
    c.lon += Math.cos(t * 0.08 + c.id.charCodeAt(2)) * 0.0004;
    c.heading = (c.heading + Math.sin(t * 0.05) * 0.5 + 360) % 360;
    if (c.speed_kts > 0) c.speed_kts += Math.sin(t * 0.2) * 2;
  }
}

function broadcast() {
  tickTactical();
  const all = [...tactical, ...flights, ...quakes, ...fires, ...weather, ...satellites, ...cctv];
  const msg = JSON.stringify({ type: "ContactUpdate", data: all });
  for (const ws of wss.clients) {
    if (ws.readyState === 1) ws.send(msg);
  }
}

setInterval(broadcast, 1000);
// refresh globe data every 2 min
refreshGlobeData();
setInterval(refreshGlobeData, 120000);

wss.on("connection", (ws) => {
  const all = [...tactical, ...flights, ...quakes, ...fires, ...weather, ...satellites, ...cctv];
  console.log(`[C2] client connected (${wss.clients.size} total) — ${all.length} contacts`);
  ws.send(JSON.stringify({ type: "ContactUpdate", data: all }));
  ws.on("close", () => console.log(`[C2] client disconnected (${wss.clients.size} total)`));
});

console.log(`[C2 BRIDGE] ws://127.0.0.1:${PORT}/c2`);
console.log("[C2 BRIDGE] layers: tactical + flights + quakes + fires + weather + satellites + cctv");
console.log("[C2 BRIDGE] globe data refreshes every 120s");
