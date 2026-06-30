# uv basics

`uv` is a fast Python package and project manager. In the lab, use it for pure Python projects, scripts, command-line tools, and projects where dependencies can be expressed cleanly in `pyproject.toml`.

## Common commands

```bash
uv init my-project
cd my-project
uv add numpy scipy matplotlib
uv sync
uv run python main.py
uv run pytest
```

## What files matter

- `pyproject.toml`: project metadata and dependency declarations.
- `uv.lock`: locked dependency versions. Commit this for reproducibility.
- `.venv/`: local virtual environment. Usually do not commit this.

## Good habits

- Commit `pyproject.toml` and `uv.lock`.
- Do not commit `.venv/`.
- Use `uv run ...` so the command runs inside the right environment.
- Prefer project-local environments over mystery global Python installs.
