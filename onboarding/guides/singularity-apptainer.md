# Singularity and Apptainer

Singularity and Apptainer are container tools commonly used on HPC clusters.

## Why not just Docker?

Docker usually requires a daemon and privileges that many shared clusters do not allow. HPC admins dislike giving every user a tiny root-shaped footgun. Reasonable, really.

## Common commands

```bash
singularity shell image.sif
singularity exec image.sif python script.py
apptainer shell image.sif
apptainer exec image.sif python script.py
```

## Binding directories

Cluster jobs often need access to project folders or scratch space:

```bash
singularity exec --bind /scratch:/scratch image.sif python run.py
```

## Practical guidance

- Use Docker locally when convenient.
- Convert or build a `.sif` image for HPC runs.
- Keep input files, commands, and image versions documented.
- Do not assume paths inside the container match paths outside the container.
