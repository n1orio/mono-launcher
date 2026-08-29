# AGENTS.md — CurseForge Description Handling (mono-launcher)

## CurseForge API v1: Full Descriptions vs Summary

**Critical:** All mods in CurseForge API `/api/v1/projects/{projectId}` return `description` field. However, for many older mods it's `null`. The website displays full HTML when available.

### What to use
- **Display order**: `p.description || p.summary` (full description first, summary as fallback)
- **Never rely on**: `summary` alone — it's a short blurb that appears secondarily on CF site

### API Response Structure
```typescript
// CurseProjectDetail (from `/api/v1/projects/{projectId}`):
interface CurseProjectDetail {
  description: string | null;  // Full HTML from CF, can be null for old mods
  summary: string;              // Short fallback text (always present)
  screenshots?: string[];      // Only on CurseProjectDetail, NOT CurseSearchHit
  ... other fields
}

// CurseSearchHit (from `/api/v1/search`):
interface CurseSearchHit {
  description: string | null;   // Full HTML from project page (not file-specific)
  summary: string;
  iconUrl?: string;             // Project logo, NOT pack-specific files
}
```

### Build Order for Launcher
```bash
# 1. Always run this first to generate .nuxt/tsconfig.json
npm run prepare

# 2. Then typecheck (needed before npm run build)
npm run typecheck

# 3. Finally build
npm run build
```

### Display Logic in Components
Use this pattern consistently:
```vue
<!-- About tab description -->
<p v-if="p.description" class="line-clamp-2">{{ p.description }}</p>
<p v-else-if="p.summary" class="mt-0.5 line-clamp-1">{{ p.summary }}</p>
```

## Key Files & Ownership
- **Frontend (Vue 3)**: `mono-launcher/` — Tauri app, uses Nuxt
- **Backend API**: `mono-launcher-backend/` — Rust + PostgreSQL  
- **Storage server**: `mono-launcher-storage/` — Rust static file server
- **Marketing site**: `mono-launcher-site/` — Vue 3 SPA

## Port Allocation
- Backend: `localhost:8080` (dev), prod host port 80/443
- Storage: `localhost:8081` (dev), separate deployment
- Launcher dev: fixed `localhost:1420`

## Environment Gotchas
```bash
# .env for backend MUST include:
STORAGE_UPLOAD_TOKEN=<same as storage UPLOAD_TOKEN>  # Required, not in .env.example
MONO_CURSEFORGE_KEY='...'   # Single-quote $ chars to prevent expansion
```
