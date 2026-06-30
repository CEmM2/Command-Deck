# Package managers

A package manager installs software and records what versions are needed.

## Why package managers matter

They help with:

- installing dependencies;
- reproducing environments;
- avoiding manual downloads;
- sharing project setup with other people;
- rebuilding work later.

## Common package managers in the lab

- `uv`: Python projects, scripts, lockfiles, tools, fast installs.
- `pip`: Python packages, usually inside an existing environment.
- `conda`: Python plus non-Python packages and compiled dependencies.
- `mamba`: faster conda-compatible solver and installer.
- `spack`: compiled scientific software, especially on HPC.
- OS package managers: `apt`, `brew`, etc. Use for system tools, not project reproducibility.

## Rule

A project should document how to create its environment. A mysterious environment that exists only on one laptop is not a research artifact.
