https://docs.solo.io/gateway/latest/quickstart/
```sh
kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.0.0/standard-install.yaml
```
```sh
helm repo add glooe https://storage.googleapis.com/gloo-ee-helm
helm repo update
```
```sh
helm install -n gloo-system gloo-gateway glooe/gloo-ee \
--create-namespace \
--version 1.17.0 \
--set-string license_key=$GLOO_LICENSE_KEY \
--set gloo.kubeGateway.enabled=true \
--set gloo.gloo.disableLeaderElection=true \
--set gloo.discovery.enabled=false \
--set observability.enabled=false \
--set prometheus.enabled=false \
--set grafana.defaultInstallationEnabled=false \
--set gloo-fed.enabled=false \
--set gloo-fed.glooFedApiserver.enabled=false \
--set gloo.gatewayProxies.gatewayProxy.disabled=true
```
```sh
kubectl apply -n gloo-system -f- <<EOF
kind: Gateway
apiVersion: gateway.networking.k8s.io/v1
metadata:
  name: http
spec:
  gatewayClassName: gloo-gateway
  listeners:
  - protocol: HTTP
    port: 8080
    name: http
    allowedRoutes:
      namespaces:
        from: All
EOF
```
```sh
kubectl create ns httpbin
kubectl -n httpbin apply -f https://raw.githubusercontent.com/solo-io/gloo-mesh-use-cases/main/policy-demo/httpbin.yaml
```
```sh
kubectl -n httpbin get pods
```
```sh
kubectl apply -f- <<EOF
apiVersion: gateway.networking.k8s.io/v1beta1
kind: HTTPRoute
metadata:
  name: httpbin
  namespace: httpbin
  labels:
    example: httpbin-route
spec:
  parentRefs:
    - name: http
      namespace: gloo-system
  hostnames:
    - "www.example.com"
  rules:
    - backendRefs:
        - name: httpbin
          port: 8000
EOF
```
```sh
export INGRESS_GW_ADDRESS=$(kubectl get svc -n gloo-system gloo-proxy-http -o jsonpath='{.status.loadBalancer.ingress[0].hostname}')
echo $INGRESS_GW_ADDRESS
```
```sh
curl -i http://$INGRESS_GW_ADDRESS:8080/headers -H "host: www.example.com:8080"
```
```sh
kubectl patch settings default -n gloo-system --type='merge' -p '{
  "spec": {
    "extProc": {
      "allowModeOverride": false,
      "failureModeAllow": false,
      "filterStage": {
        "predicate": "After",
        "stage": "AuthZStage"
      },
      "grpcService": {
        "extProcServerRef": {
          "name": "moesif-exproc-plugin",
          "namespace": "gloo-system"
        }
      },
      "processingMode": {
        "requestHeaderMode": "SEND",
        "responseHeaderMode": "SKIP"
      }
    }
  }
}'
```