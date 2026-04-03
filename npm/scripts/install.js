#!/usr/bin/env node
// Post-install script: download the correct Aether binary
const https = require('https');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const REPO = 'aether-lang/aether';
const vendorDir = path.join(__dirname, '..', 'vendor');

function getPlatform() {
    const os = process.platform;
    const arch = process.arch;
    const map = {
        'linux-x64': 'linux-x86_64',
        'linux-arm64': 'linux-aarch64',
        'darwin-x64': 'macos-x86_64',
        'darwin-arm64': 'macos-aarch64',
        'win32-x64': 'windows-x86_64',
    };
    return map[`${os}-${arch}`];
}

async function install() {
    const platform = getPlatform();
    if (!platform) {
        console.log(`No pre-built binary for ${process.platform}-${process.arch}`);
        console.log('Build from source: cargo build --release');
        return;
    }

    const ext = process.platform === 'win32' ? 'zip' : 'tar.gz';
    const artifact = `aether-${platform}`;
    const url = `https://github.com/${REPO}/releases/latest/download/${artifact}.${ext}`;

    console.log(`Downloading ${artifact}...`);

    fs.mkdirSync(vendorDir, { recursive: true });
    const tmpFile = path.join(vendorDir, `${artifact}.${ext}`);

    try {
        execSync(`curl -sSL -o "${tmpFile}" "${url}"`, { stdio: 'pipe' });

        if (ext === 'tar.gz') {
            execSync(`tar xzf "${tmpFile}" -C "${vendorDir}"`, { stdio: 'pipe' });
        } else {
            // Windows: use PowerShell to extract
            execSync(`powershell -c "Expand-Archive -Path '${tmpFile}' -DestinationPath '${vendorDir}'"`, { stdio: 'pipe' });
        }

        fs.unlinkSync(tmpFile);
        console.log(`Installed aether to ${vendorDir}`);
    } catch (e) {
        console.log('Could not download pre-built binary.');
        console.log('Install from source: cargo build --release');
    }
}

install();
