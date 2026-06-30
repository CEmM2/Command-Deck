# Docker vs Docker Compose

## Docker

Use Docker directly when you need one container:

```bash
docker build -t my-image .
docker run --rm -it my-image bash
```

Good for:

- one command-line tool;
- one reproducible runtime;
- a single simulation environment;
- a single notebook server.

## Docker Compose

Use Docker Compose when you need multiple services described together:

```bash
docker compose up
docker compose down
```

Good for:

- app plus database;
- web service plus worker;
- local stacks with several containers.

## Rule of thumb

If there is one container, Docker is enough. If there are several containers that need networking, volumes, and shared configuration, use Compose.
