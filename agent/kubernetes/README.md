# Kubernetes Deployment - CAP Verifier API v0.11.0

## 📦 Was wurde erstellt?

### Kubernetes Manifests:
- ✅ `deployment.yaml` - API Deployment (2 Replicas, Health Checks)
- ✅ `service.yaml` - ClusterIP Service (Port 8080)
- ✅ `configmap.yaml` - Environment Variables
- ✅ `pvc.yaml` - PersistentVolumeClaim (10Gi für Registry/BLOB Store)
- ✅ `serviceaccount.yaml` - ServiceAccount für Security
- ✅ `servicemonitor.yaml` - Prometheus Monitoring (Prometheus Operator)
- ✅ `kustomization.yaml` - Kustomize Config

---

## 🚀 Quick Start (Minikube)

### 1. Minikube installieren & starten
```bash
# Minikube installieren
brew install minikube

# Minikube starten
minikube start

# Status prüfen
minikube status
kubectl cluster-info
```

### 2. Docker Image in Minikube laden
```bash
# Docker Image bauen (falls noch nicht geschehen)
docker build -f Dockerfile.optimized -t cap-agent:v0.11.0-alpine .

# Image in Minikube laden
minikube image load cap-agent:v0.11.0-alpine

# Prüfen ob Image in Minikube verfügbar ist
minikube ssh -- docker images | grep cap-agent
```

### 3. Kubernetes Manifests deployen
```bash
cd /Users/tomwesselmann/Desktop/LsKG-Agent/agent

# Mit kubectl apply
kubectl apply -f kubernetes/

# ODER mit kustomize
kubectl apply -k kubernetes/

# Status prüfen
kubectl get all
kubectl get pvc
```

### 4. Service testen
```bash
# Port-Forward einrichten
kubectl port-forward svc/cap-verifier-api 8080:8080

# In neuem Terminal:
curl http://localhost:8080/healthz
curl http://localhost:8080/metrics
```

---

## 📊 Monitoring & Debugging

### Pods anzeigen
```bash
kubectl get pods
kubectl describe pod <pod-name>
```

### Logs anzeigen
```bash
# Alle Pods
kubectl logs -l app=cap-verifier-api -f

# Specific Pod
kubectl logs <pod-name> -f
```

### In Pod shell gehen
```bash
kubectl exec -it <pod-name> -- sh

# Im Pod:
ls -la /app/build
curl http://localhost:8080/healthz
```

### Metrics prüfen (mit Prometheus Operator)
```bash
# ServiceMonitor prüfen
kubectl get servicemonitor

# Prometheus UI port-forward
kubectl port-forward -n monitoring svc/prometheus-k8s 9090:9090

# Open Prometheus
open http://localhost:9090
```

---

## 🔧 Konfiguration anpassen

### Replicas ändern
```bash
kubectl scale deployment cap-verifier-api --replicas=3
```

### ConfigMap aktualisieren
```bash
# configmap.yaml editieren
kubectl apply -f kubernetes/configmap.yaml

# Pods neu starten (um neue Config zu laden)
kubectl rollout restart deployment cap-verifier-api
```

### Resource Limits anpassen
```yaml
# In deployment.yaml:
resources:
  requests:
    memory: "512Mi"
    cpu: "500m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

---

## 🏭 Production Deployment

### 1. Image zu Registry pushen
```bash
# Tag für Registry
docker tag cap-agent:v0.11.0-alpine your-registry.com/cap-agent:v0.11.0

# Push
docker push your-registry.com/cap-agent:v0.11.0
```

### 2. Deployment anpassen
```yaml
# In deployment.yaml:
spec:
  template:
    spec:
      containers:
      - name: api
        image: your-registry.com/cap-agent:v0.11.0
        imagePullPolicy: Always  # Immer neueste Version pullen
```

### 3. TLS aktivieren
```yaml
# ConfigMap:
TLS_MODE: "tls"

# Secret für Certs erstellen
kubectl create secret generic cap-tls-certs \
  --from-file=tls.crt=certs/server.crt \
  --from-file=tls.key=certs/server.key

# In Deployment Volume mount hinzufügen
volumeMounts:
- name: tls-certs
  mountPath: /certs
  readOnly: true

volumes:
- name: tls-certs
  secret:
    secretName: cap-tls-certs
```

### 4. Ingress konfigurieren
```yaml
# ingress.yaml:
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cap-verifier-api
spec:
  rules:
  - host: cap-api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cap-verifier-api
            port:
              number: 8080
```

---

## 🚨 Troubleshooting

### Pod startet nicht
```bash
# Events prüfen
kubectl describe pod <pod-name>

# Logs prüfen
kubectl logs <pod-name>

# ImagePullBackOff?
# → Image in Minikube laden: minikube image load cap-agent:v0.11.0-alpine
```

### CrashLoopBackOff
```bash
# Logs anschauen
kubectl logs <pod-name> --previous

# Health Check prüfen
kubectl exec <pod-name> -- curl http://localhost:8080/healthz
```

### PVC bleibt Pending
```bash
# PVC Status
kubectl describe pvc cap-build-pvc

# Minikube: Default StorageClass prüfen
kubectl get storageclass
```

---

## ✅ Nächste Schritte

1. [ ] Minikube installieren & starten
2. [ ] Docker Image in Minikube laden
3. [ ] Manifests deployen (`kubectl apply -k kubernetes/`)
4. [ ] Services testen (Port-Forward)
5. [ ] Prometheus Operator installieren (optional)
6. [ ] Ingress konfigurieren (optional)
7. [ ] Helm Chart erstellen (Woche 1 Tag 5-7)

---

**Erstellt:** 17. November 2025  
**Version:** v0.11.0  
**Woche 1 Tag 3-4:** Kubernetes Manifests
