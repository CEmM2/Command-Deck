# Containers basics

A container packages software and its runtime environment so it can run more consistently across machines.

## Docker

Use Docker for local development, services, and reproducible environments on machines where Docker is allowed.

Common commands:

```bash
docker build -t my-image:latest .
docker run --rm -it my-image:latest bash
docker ps
docker images
```

## Singularity/Apptainer

HPC systems often disallow Docker but allow Singularity or Apptainer. These tools are designed for shared clusters and user-level execution.

Common commands:

```bash
singularity shell image.sif
singularity exec image.sif python script.py
apptainer shell image.sif
apptainer exec image.sif python script.py
```

## What containers do not solve

Containers do not automatically solve:

- bad code;
- missing data;
- unclear instructions;
- GPU driver compatibility;
- filesystem assumptions.

So yes, documentation is still required. Humanity persists.
