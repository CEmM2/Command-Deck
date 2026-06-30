# GCP cost and safety

Cloud resources cost money while they exist or run. The fact that a command is easy to type does not mean it is cheap.

## Rules

- Stop GPU VMs when not actively using them.
- Check the active project before creating or starting resources.
- Use labels or naming conventions for lab resources.
- Do not store secrets in startup scripts, notebooks, or Git repositories.
- Avoid opening firewall rules broadly.
- Prefer SSH through normal authenticated tooling.

## Before starting a VM

```bash
gcloud config list
gcloud compute instances describe VM --zone=ZONE --format="table(name,status,machineType.basename())"
```

## After finishing work

```bash
gcloud compute instances stop VM --zone=ZONE
```

## Data warning

Stopping a VM usually keeps persistent disks. Deleting a VM or disk may remove data. Know which action you are taking before pressing the shiny button.
