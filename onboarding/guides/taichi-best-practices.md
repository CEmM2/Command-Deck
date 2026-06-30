# Taichi best practices

Taichi is useful for high-performance numerical kernels written from Python.

## Project structure

Keep a clear split between:

- configuration;
- data loading;
- Taichi kernels;
- simulation loop;
- output and visualization;
- tests or small validation scripts.

## Good habits

- Start with a small reproducible case.
- Keep units and coordinate conventions documented.
- Avoid hiding constants inside kernels.
- Save enough metadata with results to reproduce them.
- Profile before optimizing.
- Compare against a simple reference calculation when possible.

## Running

```bash
uv run python main.py
```

## Debugging

- Reduce the problem size.
- Print or save intermediate fields.
- Check boundary conditions.
- Test kernels with simple inputs.
- Confirm CPU/GPU backend assumptions.

## Lab convention

Prefer scripts that can run from a clean checkout with documented dependencies. A script that only works after three undocumented notebook cells is not a workflow.
