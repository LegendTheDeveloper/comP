#!/usr/bin/env node

/**
 * SVG to PNG icon generator for VS Code Marketplace
 *
 * Generates 128x128 PNG from resources/comp-icon.svg
 * for VS Code Marketplace compatibility
 */

const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const srcSvg = path.join(__dirname, '..', 'resources', 'comp-icon.svg');
const destPng = path.join(__dirname, '..', 'resources', 'comp-icon.png');

async function generateIcon() {
  try {
    console.log('[comP] Converting SVG icon to PNG...');

    await sharp(srcSvg)
      .resize(128, 128, {
        fit: 'contain',
        background: { r: 255, g: 255, b: 255, alpha: 1 }
      })
      .png({ quality: 100 })
      .toFile(destPng);

    console.log(`[comP] Icon generated: ${destPng}`);
    console.log('[comP] Remember to update package.json icon field to "resources/comp-icon.png"');
  } catch (error) {
    console.error('[comP] Failed to generate icon:', error.message);
    process.exit(1);
  }
}

generateIcon();
