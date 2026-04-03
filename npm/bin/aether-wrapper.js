#!/usr/bin/env node
// npx aether-lang run hello.ae
const { execFileSync } = require('child_process');
const path = require('path');
const fs = require('fs');

const binaryName = process.platform === 'win32' ? 'aether.exe' : 'aether';
const binaryPath = path.join(__dirname, '..', 'vendor', binaryName);

if (!fs.existsSync(binaryPath)) {
    console.error('Aether binary not found. Run: npm install aether-lang');
    process.exit(1);
}

try {
    execFileSync(binaryPath, process.argv.slice(2), { stdio: 'inherit' });
} catch (e) {
    process.exit(e.status || 1);
}
