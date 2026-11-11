# Karl Mark Linux — GitHub Pages Experience

A responsive, interactive desktop-inspired project landing page.

## Run locally
Open `index.html` in a browser, or use a local static server:

```bash
python -m http.server 8000
```

Then visit `http://localhost:8000`.

## Deploy to GitHub Pages
1. Copy `index.html`, `styles.css`, `script.js` and this README into the repository root (or a dedicated website folder).
2. In GitHub, open **Settings → Pages**.
3. Under **Build and deployment**, choose **Deploy from a branch**.
4. Select `main` and `/ (root)`, then Save.

Alternatively, use the included GitHub Actions workflow.

## Important
- This is a front-end concept/demo, not a real operating system running in the browser.
- Download Center links to GitHub Releases; it does not fabricate an ISO download.
- Version/build labels are demo metadata and should be kept in sync with the repository.
- Update project status and feature claims as implementation changes.

## Cursor swarm intro
The page includes an original canvas-based cursor swarm reveal. It is implemented locally in `script.js`, requires no browser extension, and does not copy/import the Swarm Rebuild repository source. Users can skip the intro; reduced-motion preferences bypass it.
