#!/usr/bin/env bash
# =============================================================================
# CAP Verifier Deployment Smoke Test
# =============================================================================
#
# Usage:
#   ./scripts/deploy-smoke-test.sh [REGISTRY] [TAG]
#
# Example:
#   ./scripts/deploy-smoke-test.sh registry.example.com/cap/verifier v1.0.0
#
# Prerequisites:
#   - Docker installed
#   - kubectl configured
#   - Secrets created (see k8s/secrets.example.yaml)
#
# =============================================================================

set -euo pipefail

# Configuration
REGISTRY="${1:-registry.example.com/cap/verifier}"
TAG="${2:-v1.0.0}"
IMAGE="${REGISTRY}:${TAG}"
NAMESPACE="${NAMESPACE:-default}"

echo "🐳 CAP Verifier Deployment Smoke Test"
echo "======================================"
echo "Image:     ${IMAGE}"
echo "Namespace: ${NAMESPACE}"
echo ""

# Step 1: Build Docker Image
echo "📦 Step 1/6: Building Docker image..."
docker build -t "${IMAGE}" .
echo "✅ Image built successfully"
echo ""

# Step 2: Push to Registry (optional, skip for local testing)
if [[ "${REGISTRY}" != "localhost"* ]]; then
  echo "📤 Step 2/6: Pushing image to registry..."
  docker push "${IMAGE}"
  echo "✅ Image pushed successfully"
else
  echo "⏭️  Step 2/6: Skipping push (local registry)"
fi
echo ""

# Step 3: Verify Image Size
echo "📊 Step 3/6: Checking image size..."
IMAGE_SIZE=$(docker images "${IMAGE}" --format "{{.Size}}")
echo "Image size: ${IMAGE_SIZE}"

# Check if image is under 100 MB (PRD requirement)
SIZE_MB=$(docker images "${IMAGE}" --format "{{.Size}}" | sed 's/MB//g' | awk '{print int($1)}')
if [[ ${SIZE_MB} -lt 100 ]]; then
  echo "✅ Image size OK (<100 MB)"
else
  echo "⚠️  Warning: Image size exceeds 100 MB target"
fi
echo ""

# Step 4: Deploy to Kubernetes
echo "☸️  Step 4/6: Deploying to Kubernetes..."
kubectl apply -f k8s/configmap.yaml -n "${NAMESPACE}"
kubectl apply -f k8s/deployment.yaml -n "${NAMESPACE}"
kubectl apply -f k8s/service.yaml -n "${NAMESPACE}"
kubectl apply -f k8s/networkpolicy.yaml -n "${NAMESPACE}"
echo "✅ Manifests applied"
echo ""

# Step 5: Wait for Pods to be Ready
echo "⏳ Step 5/6: Waiting for pods to be ready..."
kubectl wait --for=condition=ready pod -l app=cap-verifier -n "${NAMESPACE}" --timeout=120s
echo "✅ Pods are ready"
echo ""

# Step 6: Test Health Endpoint
echo "🔍 Step 6/6: Testing health endpoint..."
POD_NAME=$(kubectl get pods -l app=cap-verifier -n "${NAMESPACE}" -o jsonpath='{.items[0].metadata.name}')

echo "Pod: ${POD_NAME}"
echo ""

echo "Testing /healthz..."
kubectl exec "${POD_NAME}" -n "${NAMESPACE}" -- wget -qO- http://localhost:8443/healthz --timeout=2
echo ""

echo "Testing /readyz..."
kubectl exec "${POD_NAME}" -n "${NAMESPACE}" -- wget -qO- http://localhost:8443/readyz --timeout=2
echo ""

echo "✅ Health checks passed!"
echo ""

# Summary
echo "======================================"
echo "🎉 Smoke Test Passed!"
echo "======================================"
echo ""
echo "📋 Next Steps:"
echo "  1. Test protected endpoints with OAuth2 token"
echo "  2. Run integration tests"
echo "  3. Monitor logs: kubectl logs -l app=cap-verifier -n ${NAMESPACE}"
echo "  4. Check metrics: kubectl top pods -l app=cap-verifier -n ${NAMESPACE}"
echo ""
echo "🧹 Cleanup (when done):"
echo "  kubectl delete -f k8s/ -n ${NAMESPACE}"
echo ""
