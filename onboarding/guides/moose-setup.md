# MOOSE setup

MOOSE is a multiphysics simulation framework commonly used for finite-element applications.

## What to understand first

- MOOSE applications are usually built from source.
- Input files describe meshes, variables, kernels, materials, boundary conditions, execution, and outputs.
- Builds can depend on compilers, MPI, PETSc, libMesh, and environment setup.
- Installation details may differ between laptop, VM, and cluster.

## Basic build pattern

From a MOOSE app directory:

```bash
make -j 8
```

The exact executable name depends on the app.

## Good habits

- Record where MOOSE is installed.
- Record the commit or version.
- Keep app-specific build notes in the repository.
- Do not mix several MOOSE builds without knowing which one is active.
- Use a small input file to confirm the build before running expensive jobs.
