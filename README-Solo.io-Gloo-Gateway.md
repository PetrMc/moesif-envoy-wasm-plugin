# Moesif External Processing (ExtProc) Plugin for Solo.io Gloo Gateway

The Moesif Gloo Gateway ExtProc plugincaptures API traffic from [Solo.io Gloo Gateway](https://www.solo.io/products/gloo-gateway/) and logs it to [Moesif API Analytics and Monetization](https://www.moesif.com) platform. The implementation is architected using [Envoy External Processing filters](https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/ext_proc_filter) and tested with [Solo.io Gloo Gateway implementation of the filiter](https://docs.solo.io/gateway/latest/traffic-management/extproc/about/). 

- Gloo Gateway is a powerful ingress controller and an advanced API and AI gateway, natively supporting the Kubernetes Gateway API.
- Moesif is an API analytics and monetization platform.

[Source Code on GitHub](TO_DO)

## How to Install

### 1. Follow Gloo Gateway installation instructions

The most up to date steps to deploy Solo.io Gloo gateway can be found on this [documentation page](https://docs.solo.io/gateway/latest/quickstart/). It also includes installation of `httpbin` demo application, that will be used later in this document as an example. 

### 2. Deploy the plugin and configure the required settings of Kubernetes deplployment

Replace the value of the environmental variable with API key provided by Moesif to uniqly accociate your Gloo Gateway instance.

```bash
export MOESIF_APP_ID=your-actual-app-id
```

Apply the following manifest it will deploy the filter kuberentes deployments and service. And upstream.gloo.solo.io resources for the ExtProc filter. You might want to review the `env:` section of the deployment and adjust according to Moesif's recommandations:
Apply the following manifest it will deploy the filter Kubernetes deployments and service, and `Upstream` resources for the ExtProc filter. You might want to review the `env:` section of the deployment and adjust according to Moesif's recommendations. User and company IDs are [discussed later](#identifying-users-and-companies) in this document.

```bash
kubectl apply -f- <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: moesif-exproc-plugin
  namespace: gloo-system
spec:
  selector:
    matchLabels:
      app: moesif-exproc-plugin
  replicas: 1
  template:
    metadata:
      labels:
        app: moesif-exproc-plugin
    spec:
      containers:
        - name: moesif-exproc-plugin
          image: gcr.io/solo-test-236622/moesif-exproc-plugin:latest
          imagePullPolicy: Always
          ports:
            - containerPort: 50051
          env:
            - name: MOESIF_APPLICATION_ID
              value: $MOESIF_APP_ID # <YOUR APPLICATION ID HERE>
            - name: USER_ID_HEADER
              value: "X-User-Example-Header"  
            - name: COMPANY_ID_HEADER
              value: "X-Company-Example-Header"  
            - name: UPSTREAM
              value: "outbound|443||api.moesif.net" 
            - name: DEBUG
              value: "false" 
            - name: RUST_LOG
              value: info
---
apiVersion: v1
kind: Service
metadata:
  name: moesif-exproc-plugin
  namespace: gloo-system
  labels:
    app: moesif-exproc-plugin
  annotations:
    gloo.solo.io/h2_service: "true"
spec:
  ports:
  - port: 4445
    targetPort: 50051
    protocol: TCP
  selector:
    app: moesif-exproc-plugin
---
apiVersion: gloo.solo.io/v1
kind: Upstream
metadata:
  labels:
    app: moesif-exproc-plugin
    discovered_by: kubernetesplugin
  name: moesif-exproc-plugin
  namespace: gloo-system
spec:
  discoveryMetadata: {}
  useHttp2: true
  kube:
    selector:
      app: moesif-exproc-plugin
    serviceName: moesif-exproc-plugin
    serviceNamespace: gloo-system
    servicePort: 4445
EOF
```


### 3. Enable extProc in Gloo Gateway

To do that please apply the follwoing parameters to Gloo Gateway `settings.gloo.solo.io` custom resourse:

```bash
kubectl patch settings default -n gloo-system --type='merge' -p '{
  "spec": {
    "extProc": {
      "allowModeOverride": false,
      "failureModeAllow": true,
      "filterStage": {
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
        "responseHeaderMode": "SEND"
      }
    }
  }
}'
```

### 4. Test

Make a few API calls that pass through the Gloo Gateway. These calls should now be logged to your Moesif account.

## How to Use

###  Capturing API traffic

The Moesif plugin for Gloo Gateway captures API traffic from Envoy and logs it to Moesif automatically when Gloo Gateway routes traffic through the plugin. Gloo Gateway traffic flow is defined via [Kubernetes Gateway APIs](https://gateway-api.sigs.k8s.io/)   that allows only required traffic to be accessable by plugin. Fo

###  Identifying users and companies

This plugin will automatically identify API users so you can associate API traffic to web traffic and create cross-platform funnel reports of your customer journey. The plugin currently supports reading request headers to identify users and companies automatically from events.

- If the `user_id_header` or `company_id_header` configuration option is set, the named request header will be read from each request and it's value will be included in the Moesif event model as the `user_id` or `company_id` field respectively.
2. You can associate API users to companies for tracking account-level usage. This can be done either with the company header above or through the Moesif [update user API](https://www.moesif.com/docs/api#update-a-user) to set a `company_id` for a user. Moesif will associate the API calls automatically.

## Configuration Options

These configuration options are specified as variables in the `env:` portion of the filter kubernetes deployment. 

| Option                 | Type    | Default                 | Description                                                                                                                             |
|------------------------|---------|-------------------------|-----------------------------------------------------------------------------------------------------------------------------------------|
| `moesif_application_id`| String  | None                    | **Required.** Your Moesif Application Id. Can be found within the Moesif Portal.                                                        |
| `user_id_header`       | String  | None                    | Optional. The header key for User Id. If provided, the corresponding header value is used as the User Id in Moesif event models.        |
| `company_id_header`    | String  | None                    | Optional. The header key for Company Id. If provided, the corresponding header value is used as the Company Id in Moesif event models.  |
| `batch_max_size`       | Integer | 100                     | Optional. The maximum batch size of events to be sent to Moesif.                                                                       |
| `batch_max_wait`       | Integer | 2000                    | Optional. The maximum wait time in milliseconds before a batch is sent to Moesif, regardless of the batch size.                              |
| `upstream`             | String  | "moesif_api"            | Optional. The upstream cluster that points to Moesif's API.                                                                            |


## Example

Based on Gloo Gateway External process documentation, this example shows how the traffic that is sent to `httpbin` service gets monitored by Moesif platform via ExtProc http filter.

### Prerequisites

You'll need access to a Kubernetes cluster into which to install the Moesif Istio WASM Plugin.
- Kubernetes Cluster, cloud based or hosted cluster installation should be very similar.
- [kubectl](https://kubernetes.io/docs/tasks/tools/install-kubectl/) (v1.29 or later)


### Gloo Gateway installation.

For ExtProc support you will need the enterprise version of Gloo Gateway. If not deployed yet, after obtaining your license key, follow the instructions 

We're going to use the Istio CLI tool for the installation.

1. Install the Istio base chart which includes cluster-wide resources used by the Istio operator:

```bash
istioctl install --set profile=demo -y
```

2. Check the status of the Istio system pods:

```bash
kubectl get pods -n istio-system
```

### Deploy the Moesif Istio WASM Plugin Example

1. Navigate to the `examples/istio` directory in this repository:

2. Apply the resources to your Minikube cluster.  This will create the echo service, Istio inbound configuration to route traffic to the echo service, Istio outbound configuration to allow Istio to contact Moesif's API, and the Moesif WASM plugin itself:

```bash
kubectl apply -f echo-service.yaml \
 -f istio-echo-inbound.yaml \
 -f istio-moesif-outbound.yaml \
 -f moesif-wasm-plugin.yaml
```

3. Verify that the echo service is running:

```bash
kubectl get pods -n default
```

You should see the `echo` pod in the Running status.

4. Test the echo service:

```bash
kubectl port-forward svc/echo 8080:80
```

Now you can send a request to the echo service running inside the cluster from the host machine:

```bash
curl http://localhost:8080/echo
```

You should get a response: `Hello from echo service` validating the service is running.

### Moesif WASM Plugin Configuration

The WasmPlugin YAML definition in `moesif-wasm-plugin.yaml` configures the Moesif Istio WASM Plugin.  The pluginConfig section of the YAML definition is shown below:

```yaml
    moesif_application_id: <YOUR MOESIF APPLICATION ID>
    upstream: outbound|443||api.moesif.net
```

This configuration allows the plugin to capture and log the requests and responses flowing through the Istio service mesh. To use the plugin, you need a Moesif application id, which is set in the `moesif_application_id` field in the plugin configuration. You can get this from your Moesif dashboard.

Remember to replace the `moesif_application_id` and `upstream` values in with your actual values.  The upstream string value is the cluster name that points to Moesif's API in the Istio outbound configuration.  `debug` is set to `true` to enable debug logging for the example, but this should be set to `false` in production.

### Accessing the echo service via Istio Ingress Gateway

Next, access the echo service via Istio Ingress Gateway, you first need to determine the ingress IP and ports:

- For Minikube:

```bash
export INGRESS_HOST=$(minikube ip)
export INGRESS_PORT=$(kubectl -n istio-system get service istio-ingressgateway -o jsonpath='{.spec.ports[?(@.name=="http2")].nodePort}')
```

Now, you can send a request to the echo service through the Istio Ingress Gateway and the Moesif WASM Plugin:

```bash
curl -sS "http://${INGRESS_HOST}:${INGRESS_PORT}/echo"
```

The response should be `Hello from echo service`.

### Finished, Check Logs

After the configuration is applied, you can check the events in your https://moesif.com account to see the plugin in action.

## Other Integrations

To view more documentation on integration options, please visit __[the Integration Options Documentation](https://www.moesif.com/docs/getting-started/integration-options/).__
