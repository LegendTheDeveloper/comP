#!/usr/bin/env node

/**
 * Generate 24x24 PNG icon: circle with pentagram cutout
 * Uses SVG as intermediate format, rendered to PNG by sharp
 */

const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

// Create SVG with circle and pentagram (5-pointed star) cutout
const svgContent = `
<svg width="24" height="24" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <mask id="pentagram-mask">
      <!-- White rect = shown -->
      <rect width="24" height="24" fill="white"/>
      <!-- Black pentagram = transparent cutout -->
      <polygon points="12,5.5 13.854,8.618 17.254,9.118 14.854,11.382 15.472,14.854 12,12.618 8.528,14.854 9.146,11.382 6.746,9.118 10.146,8.618" fill="black"/>
    </mask>
  </defs>
  <!-- Black circle masked with pentagram cutout -->
  <circle cx="12" cy="12" r="10.5" fill="black" mask="url(#pentagram-mask)"/>
</svg>
`;

const destPng = path.join(__dirname, '..', 'resources', 'comp-icon.png');

async function generateIcon() {
  try {
    console.log('[comP] Generating PNG icon from SVG...');

    // Convert SVG to PNG using sharp
    await sharp(Buffer.from(svgContent))
      .resize(24, 24, {
        fit: 'contain',
        background: { r: 0, g: 0, b: 0, alpha: 0 }
      })
      .png()
      .toFile(destPng);

    console.log(`[comP] PNG icon generated: ${destPng}`);
    console.log('[comP] Size: 24x24, transparent background (RGBA)');
  } catch (error) {
    console.error('[comP] Failed to generate PNG icon:', error.message);
    process.exit(1);
  }
}

generateIcon();
