# Gemini CLI Engineering Standards

For this project, the following workflows are mandatory:

1. **Verification Loop**: Before finishing any task or committing changes, you MUST run:
   - `cargo check` to ensure code validity.
   - `cargo test` to ensure no regressions.
   - `cargo clippy -- -D warnings` to ensure high code quality.
   
2. **Auto-Fixing**: If any of the above commands fail, you MUST attempt to fix the issues (using `cargo clippy --fix` where appropriate or manual edits) and re-verify until all checks pass.

3. **Dashboard Consistency**: Any changes to dashboard logic must be reflected in `SPEC.md` if they alter the architecture or user-facing behavior.

4. **Performance**: Always prioritize efficient system data polling. Avoid heavy I/O or O(N^2) operations in the render loop.
