# MOOSE development

MOOSE development usually means editing C++ objects, input files, tests, and documentation.

## Common pieces

- **Input files**: define the simulation.
- **Kernels**: residual/Jacobian contributions for PDE terms.
- **Materials**: compute material properties.
- **BCs**: boundary conditions.
- **AuxKernels**: derived or diagnostic fields.
- **Tests**: small cases that prove behavior.

## Run an input

```bash
./app-opt -i input.i
```

With MPI:

```bash
mpiexec -n 4 ./app-opt -i input.i
```

## Good development habits

- Start from an existing similar object.
- Add or update a small test.
- Keep input files minimal while debugging.
- Check units and sign conventions.
- Search existing MOOSE examples with `rg`.
- Document assumptions in the input or source.

## Debugging order

1. Does it build?
2. Does a tiny input run?
3. Are variables and materials coupled correctly?
4. Are boundary conditions correct?
5. Does mesh/time-step refinement change the answer?
