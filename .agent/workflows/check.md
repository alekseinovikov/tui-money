---
description: Run code quality checks (clippy and fmt)
---

1. Run clippy and apply fixes
// turbo
2. Run cargo clippy --fix --workspace --allow-dirty --allow-staged

3. Format code
// turbo
4. Run cargo fmt --all

5. Verify no remaining errors
// turbo
6. Run cargo check --workspace
