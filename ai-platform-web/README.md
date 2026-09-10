# AI Platform Web

Unified web console for AI Flow: Projects, Studio, Playground, Evaluation, Monitoring, and Publish.

## Stack

- React 19 + TypeScript + Vite
- Material UI (dark theme)
- React Router, TanStack Query, Zustand
- React Flow, Monaco Editor, Axios

## Quick start

```bash
# From repo root — starts ai-api :8080 and web :3000
./ai-platform-web/platform.sh

# Or manually
cd ai-platform-web
npm install
npm run dev
```

API defaults to `http://127.0.0.1:8080`. Configure in **Settings** or via `VITE_API_URL`.

## Scripts

| Command | Description |
|---------|-------------|
| `npm run dev` | Vite dev server (port 3000) |
| `npm run build` | Production build |
| `npm run preview` | Preview production build |
| `npm test` | Vitest unit/component tests |
| `npm run test:e2e` | Playwright E2E (builds + preview on :4173) |
| `./platform.sh` | Full stack launcher |

## Navigation

- **Projects** — create, open, rename, delete projects
- **Studio** — company, knowledge, search preview, flows, prompts, clients
- **Playground** — sessions, chat, debug pipeline
- **Evaluation** — eval scenarios and runs
- **Monitoring** — runtime overview, sessions
- **Publish** — draft publish pipeline and versions
- **Settings** — API URL, debug mode

Press **Ctrl+K** for the command palette.
