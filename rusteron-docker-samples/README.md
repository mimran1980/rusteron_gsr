# Containerized Aeron Example with Podman and Kubernetes

This repository provides a minimal setup for containerizing and orchestrating Aeron-based components using **Docker**, **Podman**, and **Kubernetes**.  
While not production-ready, it serves as a clean starting point for development, testing, and integration of Aeron-based workflows.

---

## Overview

The setup includes:

1. **Aeron Media Driver** – A containerized Aeron Archive Media Driver.
2. **Ticker Writer** – Publishes simulated Binance ticker data and archives it.
3. **Ticker Reader** – Reads from both the archive and live channel, publishing periodic stats.

Together, these components demonstrate a minimal end-to-end Aeron pipeline using shared memory and volumes.

---

## Quick Start for Local Testing

### Prerequisites

* **Podman** or **Docker** installed for image building and container runtime.
* **Kubernetes** (optional) for orchestration. If using Docker Desktop, ensure Kubernetes is enabled.

### Podman (Recommended for Local Testing)

Podman is preferred for its simplicity and lack of daemon dependency.

1. **Build Podman Images**
   ```bash
   just podman-build
   # Builds all container images using Podman.
````

2. **Deploy Locally with Podman**

   ```bash
   just podman-deploy
   # Launches the Aeron media driver, writer, and reader using pod definitions.
   ```

3. **Stop and Clean Up**

   ```bash
   just podman-stop
   # Gracefully stops and removes all running containers and shared volumes.
   ```

---

### Kubernetes Deployment (Alternative)

If you have a Kubernetes cluster available (e.g. via Docker Desktop or Minikube), you can use the provided manifests.

1. **Build Docker Images**

   ```bash
   just docker-build
   # Builds all required images using Docker (for Kubernetes use).
   ```

2. **Deploy to Kubernetes**

   ```bash
   just k8s-deploy
   # Applies the Kubernetes manifests to your current cluster context.
   ```

3. **Clean Up**

   ```bash
   just k8s-clean
   # Deletes deployed Kubernetes resources and persistent volumes.
   ```

---

## Key Features Demonstrated

* **Shared Memory (`/dev/shm`)**: Enables zero-copy Aeron message passing between containers.
* **Shared Volumes (`/data`)**: Used for persistent archive storage between writer and reader.
* **Configurable Task Automation**: Easily build, deploy, and clean with `just` recipes.
* **Minimal Real-Time Feed Simulation**: Simulated publisher/reader interaction using Aeron archive APIs.

> This example is designed for experimentation and learning. For production use, additional security, orchestration policies, monitoring, and failover logic are required.