const https = require('https');
const fs = require('fs');
const pkgNames = ['hyper','reqwest','rustls','tokio','serde','serde_json','h2','chrono','time','aws-smithy-runtime','aws-config','aws-sdk-kms','dotenvy','base64','futures','ring','openssl','url','sqlx','slog','tokio-rustls','tokio-util','tokio-stream','tonic','rustls-pemfile','reqwest-middleware','reqwest-retry'];
const lock = fs.readFileSync('Cargo.lock', 'utf8');
const lines = lock.split(/\r?\n/);
let pkgs = [];
let cur = null;
for (const line of lines) {
  if (line === '[[package]]') { cur = {}; }
  else if (cur && line.startsWith('name = ')) { cur.name = line.split('=')[1].trim().slice(1, -1); }
  else if (cur && line.startsWith('version = ')) { cur.version = line.split('=')[1].trim().slice(1, -1); }
  else if (cur && line.trim() === '') { if (cur.name) pkgs.push(cur); cur = null; }
}
if (cur && cur.name) pkgs.push(cur);
const versionsByName = {};
pkgs.forEach(p => { if (!versionsByName[p.name]) versionsByName[p.name] = new Set(); versionsByName[p.name].add(p.version); });
const packageVersions = {};
Object.keys(versionsByName).forEach(name => { if (pkgNames.includes(name)) packageVersions[name] = [...versionsByName[name]]; });
const semver = {
  parse: v => { const m = v.trim().match(/^(\d+)\.(\d+)\.(\d+)(?:[-+].*)?$/); return m ? { major: +m[1], minor: +m[2], patch: +m[3] } : null; },
  cmp: (a, b) => {
    if (!a || !b) return 0;
    return a.major - b.major || a.minor - b.minor || a.patch - b.patch;
  },
  valid: v => !!v.match(/^\d+\.\d+\.\d+(?:[-+].*)?$/),
};
function satisfiable(version, range) {
  const v = semver.parse(version);
  if (!v) return false;
  range = range.trim();
  if (range === '*') return true;
  if (range.includes('||')) return range.split('||').map(r => r.trim()).some(r => satisfiable(version, r));
  if (range.includes(',')) return range.split(',').map(r => r.trim()).every(r => satisfiable(version, r));
  if (range.startsWith('>=')) { const other = semver.parse(range.slice(2).trim()); return semver.cmp(v, other) >= 0; }
  if (range.startsWith('>')) { const other = semver.parse(range.slice(1).trim()); return semver.cmp(v, other) > 0; }
  if (range.startsWith('<=')) { const other = semver.parse(range.slice(2).trim()); return semver.cmp(v, other) <= 0; }
  if (range.startsWith('<')) { const other = semver.parse(range.slice(1).trim()); return semver.cmp(v, other) < 0; }
  if (range.startsWith('^')) {
    const other = semver.parse(range.slice(1).trim()); if (!other) return false;
    if (other.major === 0) {
      if (v.major !== 0) return false;
      if (v.minor !== other.minor) return false;
      return semver.cmp(v, other) >= 0;
    }
    if (v.major !== other.major) return false;
    if (v.minor < other.minor) return false;
    if (v.minor === other.minor) return v.patch >= other.patch;
    return true;
  }
  if (range.startsWith('~')) {
    const other = semver.parse(range.slice(1).trim()); if (!other) return false;
    if (v.major !== other.major) return false;
    if (v.minor !== other.minor) return false;
    return semver.cmp(v, other) >= 0;
  }
  if (semver.valid(range)) return semver.cmp(v, semver.parse(range)) === 0;
  return false;
}
function fetchJson(url) {
  return new Promise((resolve, reject) => {
    https.get(url, { headers: { 'User-Agent': 'node' } }, res => {
      let d = '';
      res.on('data', c => d += c);
      res.on('end', () => resolve(JSON.parse(d)));
    }).on('error', reject);
  });
}
function fetchText(url) {
  return new Promise((resolve, reject) => {
    https.get(url, { headers: { 'User-Agent': 'node' } }, res => {
      let d = '';
      res.on('data', c => d += c);
      res.on('end', () => resolve(d));
    }).on('error', reject);
  });
}
(async () => {
  const tree = await fetchJson('https://api.github.com/repos/RustSec/advisory-db/git/trees/main?recursive=1');
  const paths = tree.tree.map(x => x.path).filter(p => p.startsWith('crates/') && p.endsWith('.md'));
  const results = [];
  for (const pkg of Object.keys(packageVersions)) {
    const relevant = paths.filter(p => p.startsWith('crates/' + pkg + '/'));
    for (const path of relevant) {
      const content = await fetchText('https://raw.githubusercontent.com/RustSec/advisory-db/main/' + path);
      const patchedMatch = content.match(/patched\s*=\s*\[([^\]]*)\]/m);
      const unaffectedMatch = content.match(/unaffected\s*=\s*\[([^\]]*)\]/m);
      const patched = patchedMatch ? patchedMatch[1].split(',').map(s => s.replace(/['"\s]/g, '')).filter(Boolean) : [];
      const unaffected = unaffectedMatch ? unaffectedMatch[1].split(',').map(s => s.replace(/['"\s]/g, '')).filter(Boolean) : [];
      for (const version of packageVersions[pkg]) {
        const isPatched = patched.length && patched.some(r => satisfiable(version, r));
        const isUnaffected = unaffected.length && unaffected.some(r => satisfiable(version, r));
        if (!isPatched && !isUnaffected) {
          results.push({ pkg, version, path, patched, unaffected });
        }
      }
    }
  }
  console.log(JSON.stringify(results, null, 2));
})();
