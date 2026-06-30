# uv vs conda

Use the simplest tool that can reproduce the environment.

## Use `uv` when

- the project is mostly or entirely Python;
- dependencies are available as Python wheels or normal Python packages;
- you want fast installs and a lockfile;
- you are writing scripts, packages, tests, or lightweight tools.

## Use conda/mamba when

- you need non-Python compiled dependencies;
- the stack depends on CUDA, MPI, MKL, PETSc, VTK, or similar libraries;
- the project has known conda-forge recipes;
- you need an environment that includes system-like libraries without touching the OS.

## Use containers when

- the full OS-level environment matters;
- you need to ship or freeze a complete runtime;
- you need the same workflow on another machine or cluster;
- dependency installation has become a ritual sacrifice.

## A practical lab rule

Start with `uv` for Python-only projects. Move to `mamba` when compiled scientific dependencies become the main problem. Use containers when the whole runtime must be reproducible.
