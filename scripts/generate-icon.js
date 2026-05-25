#!/usr/bin/env node

/**
 * SVG to PNG icon generator for VS Code Marketplace
 *
 * Generates 128x128 PNG: white background, black circle, precise black pentagram
 */

const sharp = require('sharp');
const path = require('path');

const destPng = path.join(__dirname, '..', 'resources', 'comp-icon.png');

// SVG: white background + circle outline + pentagram outline (stroke-based)
// Pentagram sized to touch the circle (outer radius 56)
const svgContent = `
<svg width="128" height="128" viewBox="0 0 128 128" xmlns="http://www.w3.org/2000/svg">
  <!-- White background -->
  <rect width="128" height="128" fill="white"/>

  <!-- Black circle outline (stroke only) -->
  <circle cx="64" cy="64" r="60" fill="none" stroke="black" stroke-width="3"/>

  <!-- Black pentagram outline (5-pointed star, one-stroke style, touching circle) -->
  <!-- Points: 0(top) → 2(right-bottom) → 4(left-top) → 1(right-top) → 3(left-bottom) → 0 -->
  <polyline points="64,8 96.91,109.28 10.76,46.72 117.24,46.72 31.09,109.28 64,8" fill="none" stroke="black" stroke-width="3" stroke-linejoin="round"/>
</svg>
`;

async function generateIcon() {
  try {
    console.log('[comP] Generating Marketplace PNG icon...');

    await sharp(Buffer.from(svgContent))
      .resize(128, 128, {
        fit: 'contain',
        background: { r: 255, g: 255, b: 255, alpha: 1 }
      })
      .png()
      .toFile(destPng);

    console.log(`[comP] Marketplace icon generated: ${destPng}`);
    console.log('[comP] 128x128, white background, black circle, precise pentagram');
  } catch (error) {
    console.error('[comP] Failed to generate icon:', error.message);
    process.exit(1);
  }
}

generateIcon();
