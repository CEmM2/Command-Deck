# GCP VM selection guide

Choose the smallest machine that can do the job reliably.

## Questions to answer first

- Is this CPU-bound, memory-bound, GPU-bound, or IO-bound?
- How long will it run?
- How large is the data?
- Does it need persistent disk?
- Does it need a GPU?
- Can it run locally or on the cluster instead?

## Rough choices

- Light terminal work: small general-purpose VM.
- CPU simulations: general-purpose or compute-optimized VM.
- Memory-heavy preprocessing: high-memory VM.
- GPU workloads: GPU VM with the minimum GPU class that supports the job.
- Persistent service: only if someone owns uptime, updates, and costs.

## Cost sanity

- Stop VMs when idle.
- Delete unused disks and snapshots only after confirming data is backed up.
- Do not attach GPUs as a default.
- Record why a machine size was chosen.

Cloud bills are very efficient feedback mechanisms.
