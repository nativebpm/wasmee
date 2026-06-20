# Task Checklist: WASMEE-105 (Install Script & Packaging)

- [x] Create `site/public/bin/` folder and copy Darwin ARM64 binary.
- [x] Compile Linux ARM64 binary inside Docker and copy to `site/public/bin/wasmee-linux-arm64`.
- [x] Compile Linux AMD64 binary inside Docker and copy to `site/public/bin/wasmee-linux-amd64`.
- [x] Create `site/public/install.sh` installation script.
- [x] Build the static site and confirm files are bundled in `site/dist/`.
- [x] Test installation script inside Ubuntu and Alpine Docker containers (verified all tests passed!).
- [x] Commit and push changes to Git to trigger CI/CD pipeline deployment.
- [x] Document changes in walkthrough.md.
