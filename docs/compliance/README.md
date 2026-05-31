# Compliance

## CRA (EU Cyber Resilience Act) — Applicability

comP is free, open-source software distributed at no charge with no commercial support offering.
Under Regulation (EU) 2024/2847 Art. 2(5)(a), non-commercial open-source software developed and
supplied outside the course of a commercial activity is **exempt** from CRA manufacturer obligations.

**Evidence of exemption:**

- License: MIT (permissive, no fee)
- Distribution: GitHub Releases (free download)
- Revenue model: none
- Support: community/voluntary via GitHub Issues

If comP is incorporated into a commercial product, the downstream manufacturer is responsible
for CRA compliance of their product. They may request the SBOM from the latest GitHub Release.

## SBOM

Machine-readable Software Bill of Materials (CycloneDX 1.6 JSON) is generated automatically
at each release and attached to the GitHub Release as `sbom.cdx.json`.

Download: `https://github.com/tsucky230/comP/releases/latest`

## VEX

`vex.cdx.json` in this directory declares the exploitability status of known CVEs in comP's
dependency tree. Updated when new CVEs are published that affect listed components.

## Vulnerability Reporting

See [SECURITY.md](../../SECURITY.md) at the repository root.
