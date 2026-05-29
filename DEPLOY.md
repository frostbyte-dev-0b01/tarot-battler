# Deploying the dev tool to GitHub Pages

The Tarot Battler dev tool (`tools/ui/`) is a fully static site: the battle
engine is compiled to WebAssembly and committed (`tools/ui/engine/`), the UI is
plain HTML/CSS/JS, and content data is read from the embedded WASM, so there is
no build step and no backend.

## One-time setup

In the repository: **Settings → Pages → Build and deployment → Source: Deploy
from a branch → Branch: `main`, folder `/ (root)`**. Pages then republishes on
every push to `main`. No build step is needed — the engine WASM is committed
and there is nothing to compile.

The site publishes the repo root, so:

- `https://<owner>.github.io/<repo>/` redirects to the app (root `index.html`).
- The app itself lives at `https://<owner>.github.io/<repo>/tools/ui/index.html`.

`.nojekyll` is present so Pages serves all files as-is (no Jekyll processing).

## Rebuilding the engine

If you change the Rust engine or its content data, rebuild the committed WASM so
the deployed site stays in sync:

```bash
tools/ui/build-engine.sh
```

(One-time prerequisites: `rustup target add wasm32-unknown-unknown` and
`cargo install wasm-bindgen-cli`.) Commit the regenerated `tools/ui/engine/`.

## Playing with a friend (no backend)

1. Each player opens the Pages URL and builds teams (saved to the browser via
   localStorage; the team roster persists across reloads).
2. Export a team as JSON (💾) and share the file.
3. Import the opponent's team (📂), save it to your roster, then fight it in the
   **Training Arena** — single battle for a shared replay, or many battles for a
   win rate with a confidence interval.

Because battles are deterministic for a given seed, both players see the same
result for the same two teams.
