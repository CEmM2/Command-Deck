# GCP basics

Google Cloud Platform is a cloud provider. In the lab, it is mainly useful for virtual machines, GPUs, storage, and occasional managed services.

## Core terms

- **Project**: billing, permissions, and resource container.
- **Zone**: physical/region-specific location for resources.
- **VM instance**: a virtual machine.
- **Disk**: persistent storage attached to a VM.
- **Machine type**: CPU and memory shape.
- **GPU**: optional accelerator attached to a VM.

## Common commands

```bash
gcloud auth list
gcloud config list
gcloud config set project PROJECT_ID
gcloud compute instances list
gcloud compute ssh VM_NAME --zone=ZONE
```

## Basic workflow

1. Confirm the active account and project.
2. List VMs.
3. Start only what you need.
4. SSH in.
5. Run work.
6. Copy results out.
7. Stop expensive machines.

That last step is not decorative.
