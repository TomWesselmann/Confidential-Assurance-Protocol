# 🐳 PRD – Docker/K8s-Container für CAP Verifier

**Ziel:** Reproduzierbare, gehärtete Container-Runtime für On-prem & SAP BTP (Kyma).  
**Scope:** Build, Hardening, Config, Probes, Helm-Chart, Deployment-Checkliste.  
**Sicherheit:** non-root, read-only FS, minimal Base, signierte Images, mTLS-Secrets.

---

## 📦 Image-Build (Dockerfile Referenz)

```dockerfile
# syntax=docker/dockerfile:1.7

FROM rust:1.81-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release && strip /src/target/release/verifier

FROM gcr.io/distroless/cc-debian12:nonroot
USER nonroot:nonroot
WORKDIR /app

COPY --from=build /src/target/release/verifier /app/verifier
COPY config /app/config
COPY openapi /app/openapi

EXPOSE 8443
ENTRYPOINT ["/app/verifier","--config","/app/config/app.yaml"]
```

---

## 🔒 Container-Hardening Checkliste

- läuft **als non-root**
- **ReadOnlyRootFilesystem** in K8s aktiv
- **dropCapabilities: ALL**
- **seccomp: default**, **no privilege escalation**
- keine ausgehenden Verbindungen (NetworkPolicy)
- **SBOM** erzeugen (z. B. `syft . -o spdx-json=sbom.json`)
- Image **signieren** (cosign) + **Policy** (Kyverno)
- **Probes** bereit (healthz/readyz)
- **Resource Limits** gesetzt

---

## ⚙️ Konfiguration (12-Factor)

**Runtime-Options (ENV / Flags):**
```
SERVER_BIND=0.0.0.0:8443
CONFIG_PATH=/app/config/app.yaml
OAUTH_ISSUER=...
OAUTH_AUDIENCE=cap-verifier
MTLS_CA_PATH=/etc/mtls/ca.crt
TLS_CERT_PATH=/etc/tls/tls.crt
TLS_KEY_PATH=/etc/tls/tls.key
LOG_LEVEL=info
```

**Mounts (K8s):**
- `ConfigMap` → `/app/config/app.yaml`
- `Secret` (TLS/mTLS) → `/etc/tls`, `/etc/mtls`
- `Secret` (Ed25519 key) → `/etc/keys/agent.ed25519`

---

## ☸️ Kubernetes Manifeste (Minimal)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cap-verifier
spec:
  replicas: 2
  selector: {matchLabels: {app: cap-verifier}}
  template:
    metadata:
      labels: {app: cap-verifier}
    spec:
      securityContext:
        runAsNonRoot: true
        seccompProfile: {type: RuntimeDefault}
      containers:
        - name: verifier
          image: registry.example.com/cap/verifier:v1.0.0
          ports: [{containerPort: 8443, name: https}]
          resources:
            requests: {cpu: "100m", memory: "128Mi"}
            limits:   {cpu: "500m", memory: "512Mi"}
          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            capabilities: {drop: ["ALL"]}
          volumeMounts:
            - {name: cfg,  mountPath: /app/config}
            - {name: tls,  mountPath: /etc/tls, readOnly: true}
            - {name: mtls, mountPath: /etc/mtls, readOnly: true}
            - {name: keys, mountPath: /etc/keys, readOnly: true}
          readinessProbe:
            httpGet: {path: /readyz, port: https, scheme: HTTPS}
          livenessProbe:
            httpGet: {path: /healthz, port: https, scheme: HTTPS}
      volumes:
        - name: cfg
          configMap: {name: cap-verifier-config}
        - name: tls
          secret: {secretName: cap-verifier-tls}
        - name: mtls
          secret: {secretName: cap-verifier-mtls}
        - name: keys
          secret: {secretName: cap-agent-key}
---
apiVersion: v1
kind: Service
metadata:
  name: cap-verifier
spec:
  selector: {app: cap-verifier}
  ports:
    - name: https
      port: 443
      targetPort: https
      protocol: TCP
```

---

## 📊 Observability

- **Logs:** JSON-structured  
- **Metrics:** `/metrics` (Prometheus, intern)  
- **Traces:** optional OTLP (ohne PII)  
- **Probes:** `/healthz` (liveness), `/readyz` (readiness)

---

## 🔐 Secrets & Keys

- TLS Server-Zertifikat: `Secret cap-verifier-tls`  
- mTLS CA/Client: `Secret cap-verifier-mtls`  
- Ed25519-Key: `Secret cap-agent-key` (readOnly)

---

## 🧪 Deployment-Smoke

```bash
docker build -t registry.example.com/cap/verifier:v1.0.0 .
docker push registry.example.com/cap/verifier:v1.0.0
cosign sign --key cosign.key registry.example.com/cap/verifier:v1.0.0

kubectl apply -f k8s/
kubectl get pods
kubectl logs deploy/cap-verifier
kubectl exec deploy/cap-verifier -- wget -qO- https://localhost:8443/healthz --no-check-certificate
```

---

## ✅ Abnahmekriterien (DoD)

1. Image ≤ 100 MB, **non-root**, distroless  
2. Health & Readiness Probes = 200 OK  
3. Resource Limits & NetworkPolicy aktiv  
4. Secrets korrekt gemountet, TLS/mTLS funktioniert  
5. Helm-Chart installierbar ohne manuelle Schritte  
6. SBOM erzeugt & Image signiert  
7. Smoke-Test `/verify` deterministisch, keine PII in Logs

---

**Ergebnis:**  
Produktionsreifes, gehärtetes Container‑Deployment – **bereit für BASF/EuroDat on‑prem Integration.**
