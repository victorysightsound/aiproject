#!/usr/bin/env node

const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const zlib = require('zlib');

const VERSION = '1.7.25';
const REPO = 'victorysightsound/aiproject';

function getPlatformTarget() {
  const platform = process.platform;
  const arch = process.arch;

  const targets = {
    'darwin-x64': 'x86_64-apple-darwin',
    'darwin-arm64': 'aarch64-apple-darwin',
    'linux-x64': 'x86_64-unknown-linux-gnu',
    'linux-arm64': 'aarch64-unknown-linux-gnu',
    'win32-x64': 'x86_64-pc-windows-msvc',
  };

  const key = `${platform}-${arch}`;
  const target = targets[key];

  if (!target) {
    console.error(`Unsupported platform: ${key}`);
    process.exit(1);
  }

  return target;
}

function download(url) {
  return new Promise((resolve, reject) => {
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        download(response.headers.location).then(resolve).catch(reject);
        return;
      }

      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }

      const chunks = [];
      response.on('data', (chunk) => chunks.push(chunk));
      response.on('end', () => resolve(Buffer.concat(chunks)));
      response.on('error', reject);
    }).on('error', reject);
  });
}

async function install() {
  const target = getPlatformTarget();
  const isWindows = process.platform === 'win32';
  const ext = isWindows ? 'zip' : 'tar.gz';
  const url = `https://github.com/${REPO}/releases/download/v${VERSION}/proj-${target}.${ext}`;

  console.log(`Downloading proj for ${target}...`);

  const binDir = path.join(__dirname, '..', 'bin');
  const binPath = path.join(binDir, isWindows ? 'proj.exe' : 'proj');

  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  try {
    const data = await download(url);

    if (isWindows) {
      // Extract zip on Windows using PowerShell
      const tempZip = path.join(binDir, 'temp.zip');
      fs.writeFileSync(tempZip, data);
      try {
        execSync(`powershell -Command "Expand-Archive -Path '${tempZip}' -DestinationPath '${binDir}' -Force"`, { stdio: 'inherit' });
        fs.unlinkSync(tempZip);
      } catch (e) {
        console.error('Failed to extract zip. Please extract manually from:', url);
        fs.unlinkSync(tempZip);
        process.exit(1);
      }
    } else {
      // Extract tar.gz
      const tempTar = path.join(binDir, 'temp.tar');
      const decompressed = zlib.gunzipSync(data);
      fs.writeFileSync(tempTar, decompressed);
      execSync(`tar -xf "${tempTar}" -C "${binDir}"`, { stdio: 'inherit' });
      fs.unlinkSync(tempTar);
      fs.chmodSync(binPath, 0o755);
    }

    console.log('proj installed successfully!');
  } catch (err) {
    console.error('Failed to install proj:', err.message);
    console.error('You may need to install manually from:');
    console.error(`  https://github.com/${REPO}/releases`);
    process.exit(1);
  }
}

install();
