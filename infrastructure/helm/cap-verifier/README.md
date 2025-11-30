# CAP Verifier Helm Chart

Production-ready Helm chart for deploying the CAP Verifier REST API on Kubernetes.

## Prerequisites

- Kubernetes 1.25+
- Helm 3.10+
- Container image built and pushed to registry
- Secrets created (TLS, mTLS, Ed25519 keys)

## Installation

### 1. Create Secrets

Before installing the chart, create required secrets:

```bash
# TLS Certificate (Self-Signed for Testing)
openssl req -x509 -newkey rsa:4096 -keyout tls.key -out tls.crt -days 365 -nodes -subj "/CN=cap-verifier"
kubectl create secret tls cap-verifier-tls --cert=tls.crt --key=tls.key

# mTLS CA Certificate
openssl req -x509 -newkey rsa:4096 -keyout ca.key -out ca.crt -days 3650 -nodes -subj "/CN=CAP-CA"
kubectl create secret generic cap-verifier-mtls --from-file=ca.crt=ca.crt

# Ed25519 Keys (using cap-agent)
cargo run -- sign keygen --dir keys
kubectl create secret generic cap-agent-key \
  --from-file=agent.ed25519=keys/company.ed25519 \
  --from-file=agent.pub=keys/company.pub
```

### 2. Install Chart

```bash
# Install with default values
helm install cap-verifier ./helm/cap-verifier

# Install with custom values
helm install cap-verifier ./helm/cap-verifier -f custom-values.yaml

# Install in specific namespace
helm install cap-verifier ./helm/cap-verifier --namespace cap-system --create-namespace
```

### 3. Verify Deployment

```bash
# Check pod status
kubectl get pods -l app.kubernetes.io/name=cap-verifier

# Check service
kubectl get svc cap-verifier

# Test health endpoint
kubectl port-forward svc/cap-verifier 8443:443
curl -k http://localhost:8443/healthz
```

## Configuration

See `values.yaml` for all configuration options. Key parameters:

### Image Configuration

```yaml
image:
  repository: registry.example.com/cap/verifier
  tag: "v1.0.0"
  pullPolicy: IfNotPresent
```

### Replica & Scaling

```yaml
replicaCount: 2

autoscaling:
  enabled: true
  minReplicas: 2
  maxReplicas: 10
  targetCPUUtilizationPercentage: 80
```

### OAuth2 Configuration

```yaml
config:
  oauth:
    issuer: "https://auth.example.com"
    audience: "cap-verifier"
    requiredScopes:
      - "verify:read"
```

### Resource Limits

```yaml
resources:
  limits:
    cpu: 500m
    memory: 512Mi
  requests:
    cpu: 100m
    memory: 128Mi
```

### Network Policy

```yaml
networkPolicy:
  enabled: true
  # Ingress/Egress rules configured in values.yaml
```

### Ingress

```yaml
ingress:
  enabled: true
  className: "nginx"
  hosts:
    - host: cap-verifier.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: cap-verifier-tls
      hosts:
        - cap-verifier.example.com
```

## Upgrading

```bash
# Upgrade with new image
helm upgrade cap-verifier ./helm/cap-verifier --set image.tag=v1.1.0

# Upgrade with new values
helm upgrade cap-verifier ./helm/cap-verifier -f updated-values.yaml
```

## Uninstalling

```bash
# Delete release
helm uninstall cap-verifier

# Delete secrets (if no longer needed)
kubectl delete secret cap-verifier-tls cap-verifier-mtls cap-agent-key
```

## Production Checklist

- [ ] Use production-grade container registry
- [ ] Configure proper TLS certificates (Let's Encrypt, cert-manager)
- [ ] Set resource limits based on load testing
- [ ] Enable horizontal pod autoscaling
- [ ] Configure network policies
- [ ] Set up monitoring (Prometheus/Grafana)
- [ ] Configure log aggregation (ELK, Loki)
- [ ] Enable pod disruption budgets
- [ ] Configure backup for secrets
- [ ] Set up OAuth2 with real IdP (Keycloak, Auth0)

## Troubleshooting

### Pods not starting

```bash
# Check pod events
kubectl describe pod <pod-name>

# Check logs
kubectl logs <pod-name>

# Check secrets
kubectl get secrets
```

### Health checks failing

```bash
# Port-forward and test manually
kubectl port-forward svc/cap-verifier 8443:443
curl http://localhost:8443/healthz

# Check probe configuration
kubectl describe deployment cap-verifier
```

### Network connectivity issues

```bash
# Check network policy
kubectl get networkpolicy

# Test from debug pod
kubectl run debug --rm -it --image=alpine/curl -- /bin/sh
```

## Support

For issues and questions:
- GitHub: https://github.com/yourorg/cap-agent/issues
- Email: cap-team@example.com
