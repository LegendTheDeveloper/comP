#!/usr/bin/env node

/**
 * Generate Activity Bar icon: precise pentagram (5-pointed star)
 * 24x24 PNG, transparent background, black star
 */

const sharp = require('sharp');
const path = require('path');

const destActivityBarPng = path.join(__dirname, '..', 'resources', 'comp-icon-activitybar.png');
const destActivityBarSvg = path.join(__dirname, '..', 'resources', 'comp-icon-activitybar.svg');

// SVG: circle outline + one-stroke pentagram (both outline only)
// VS Code Activity Bar PNG icons are processed as monochrome masks.
// At 24x24 viewBox, sharp's SVG renderer produces too-thin strokes,
// resulting in low alpha values that VS Code treats as nearly transparent.
// Fix: render SVG at 96x96 with thick strokes, then downsample to 24x24.
// This preserves stroke alpha through anti-aliasing.
// SVG for VS Code activity bar: uses currentColor so VS Code can apply theme color
const svgContent = `
<svg width="96" height="96" viewBox="0 0 96 96" xmlns="http://www.w3.org/2000/svg">
  <!-- Circle outline (thick stroke for clear visibility after downsampling) -->
  <circle cx="48" cy="48" r="42" fill="none" stroke="currentColor" stroke-width="5"/>

  <!-- One-stroke 5-pointed star (outline only) -->
  <!-- Outer radius 40, center (48,48), points in skip-one order: 0→2→4→1→3→0 -->
  <polyline points="48,8 71.51,78.36 9.96,33.64 86.04,33.64 24.49,78.36 48,8" fill="none" stroke="currentColor" stroke-width="5" stroke-linejoin="round"/>
</svg>
`;

// SVG for PNG rendering via sharp: needs a concrete color
const svgForPng = `
<svg width="96" height="96" viewBox="0 0 96 96" xmlns="http://www.w3.org/2000/svg">
  <circle cx="48" cy="48" r="42" fill="none" stroke="black" stroke-width="5"/>
  <polyline points="48,8 71.51,78.36 9.96,33.64 86.04,33.64 24.49,78.36 48,8" fill="none" stroke="black" stroke-width="5" stroke-linejoin="round"/>
</svg>
`;

async function generateIcon() {
  try {
    console.log('[comP] Generating Activity Bar icon (precise pentagram)...');

    // Save as SVG
    const fs = require('fs');
    fs.writeFileSync(destActivityBarSvg, svgContent.trim());
    console.log(`[comP] Activity Bar SVG icon generated: ${destActivityBarSvg}`);

    // Save as PNG
    await sharp(Buffer.from(svgForPng))
      .resize(24, 24, {
        fit: 'contain',
        background: { r: 0, g: 0, b: 0, alpha: 0 }
      })
      .png()
      .toFile(destActivityBarPng);

    console.log(`[comP] Activity Bar icon generated: ${destActivityBarPng}`);
    console.log('[comP] 24x24, transparent background, precise pentagram');
  } catch (error) {
    console.error('[comP] Failed to generate Activity Bar icon:', error.message);
    process.exit(1);
  }
}

generateIcon();
