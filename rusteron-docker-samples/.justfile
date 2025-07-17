# =============================================================================
# Dev Utilities
# =============================================================================

# Show all available tasks
list:
  cargo update
  just --list

# Automatically format and fix Clippy warnings
fix:
  cd rusteron-dummy-example && cargo fmt --all && cd -
  cd rusteron-dummy-example && cargo clippy --allow-dirty --allow-staged --fix && cd -

# =============================================================================
# Docker / Podman Builds
# =============================================================================

# Build containers using Docker
docker-build:
  docker build -t aeron-media-driver aeron-media-driver
  docker build -t rusteron-dummy-example rusteron-dummy-example

# Build containers using Podman
podman-build:
  podman build -t rusteron-dummy-example -f Dockerfile rusteron-dummy-example
  podman build -t aeron-media-driver -f Dockerfile aeron-media-driver

# =============================================================================
# Podman / K8s Deployment
# =============================================================================

# Deploy the dummy-example pod using Podman
podman-deploy:
  just podman-stop || true
  podman play kube pod.yml

# Stop and remove the dummy-example Podman pod
podman-stop:
  podman pod stop dummy-example
  podman pod rm dummy-example --force

# Deploy the dummy-example pod to Kubernetes
k8s-deploy:
  kubectl apply -f pod.yml

# Delete the dummy-example pod from Kubernetes
k8s-clean:
  kubectl delete pod dummy-example

# =============================================================================
# Runtime
# =============================================================================

# Run the Aeron Archive Driver manually using Java
run-aeron-archive-driver:
  java \
    --add-opens java.base/jdk.internal.misc=ALL-UNNAMED \
    -javaagent:../rusteron-archive/aeron/aeron-agent/build/libs/aeron-agent-1.48.4.jar \
    -cp ../rusteron-archive/aeron/aeron-all/build/libs/aeron-all-1.48.4.jar:../rusteron-archive/aeron/aeron-archive/build/libs/aeron-archive-1.48.4.jar \
    -Daeron.archive.dir=target/aeron/archive
