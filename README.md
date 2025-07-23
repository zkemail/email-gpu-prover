# ZK Email GPU Prover

A GPU-accelerated service for generating zero-knowledge proofs for the ZK Email protocol.

## Overview

This service provides an API for generating zero-knowledge proofs using GPU acceleration via CUDA. It's built using Rust and integrates with the Rapidsnark prover for efficient proof generation.

## Features

- GPU-accelerated ZK proof generation
- RESTful API for proof requests
- Caching of circuit components and proving keys
- Automatic artifact cleanup
- Kubernetes deployment configuration
- API key authentication

## Prerequisites

- NVIDIA GPU with CUDA support
- Rust and Cargo
- CUDA 12.2 or higher
- Docker (for containerized deployment)
- Unzip

## Environment Variables

- `PORT`: Server port (default: 3000)
- `API_KEY`: Required API key for authentication

## Installation

### Using Docker

```bash
# Build the Docker image
docker build -t zkemail-gpu-prover .

# Run the container
docker run -d \
  -p 3000:3000 \
  -e API_KEY=your_api_key \
  --gpus all \
  zkemail-gpu-prover
```

### Manual Installation

1. Install Rust and required dependencies:

```bash
apt-get update && apt-get install -y cmake build-essential pkg-config \
  libssl-dev libgmp-dev libffi-dev libsodium-dev nasm git \
  nodejs npm curl unzip
```

2. Install Rapidsnark:

```bash
git clone https://github.com/Orbiter-Finance/rapidsnark.git
cd rapidsnark
git submodule init
git submodule update
./build_gmp.sh host
mkdir build_prover
cd build_prover
cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package -DNVML_LIBRARY=/usr/local/cuda-12.2/targets/x86_64-linux/lib/stubs/libnvidia-ml.so
make -j$(nproc) && make install
chmod +x ../package/bin/prover_cuda
ln -s /path/to/rapidsnark/package/bin/prover_cuda /usr/local/bin/prover
```

3. Clone and build the project:

```bash
git clone https://github.com/zkemail/email-gpu-prover.git
cd email-gpu-prover
cargo build --release
```

4. Create a `.env` file:

```
API_KEY=your_secure_api_key
PORT=3000
```

5. Run the server:

```bash
cargo run --release
```

## API Documentation

### Health Check

```
GET /api/healthz
```

Returns a simple JSON response to check if the server is running.

**Response:**

```json
{
  "status": "success",
  "message": "Hello from ZK Email!"
}
```

### Generate Proof

```
POST /api/prove
```

Generates a zero-knowledge proof for the provided input.

**Headers:**

- `x-api-key`: Your API key (required)
- `Content-Type`: application/json

**Request Body:**

```json
{
  "blueprintId": "string", // client generated random ID. Should be unique for each circuit (zkey and cpp witness generator)
  "proofId": "string",
  "input": {
    // Circuit-specific input data (JSON object)
  },
  "zkeyDownloadUrl": "string",
  "circuitCppDownloadUrl": "string"
}
```

**Response:**

```json
{
  "proof": {
    "pi_a": ["string", "string", "string"],
    "pi_b": [
      ["string", "string"],
      ["string", "string"],
      ["string", "string"]
    ],
    "pi_c": ["string", "string", "string"],
    "protocol": "string"
  },
  "publicOutputs": ["string", "string", "..."]
}
```

## Kubernetes Deployment

The repository includes Kubernetes configuration files for GKE deployment. Follow these steps to deploy to your GKE cluster:

### Prerequisites

- Google Kubernetes Engine (GKE) cluster with GPU node pools
- kubectl configured with your cluster context
- A static IP address reserved in Google Cloud
- Domain DNS pointing to the static IP

### Deployment Steps

1. **Create the PersistentVolumeClaim:**
   ```bash
   kubectl apply -f - --context=your-gke-context <<EOF
   apiVersion: v1
   kind: PersistentVolumeClaim
   metadata:
     name: prover-pvc
     namespace: sdk
   spec:
     accessModes:
       - ReadWriteOnce
     resources:
       requests:
         storage: 50Gi
   EOF
   ```

2. **Create the Kubernetes Secret with API key:**
   ```bash
   kubectl create secret generic k8s-gpu-prover-secret \
     --from-literal=API_KEY=your-api-key \
     --namespace=sdk \
     --context=your-gke-context
   ```

3. **Create the Service:**
   ```bash
   kubectl apply -f - --context=your-gke-context <<EOF
   apiVersion: v1
   kind: Service
   metadata:
     name: k8s-gpu-prover-svc
     namespace: sdk
   spec:
     selector:
       app: prover
     ports:
       - protocol: TCP
         port: 443
         targetPort: 3000
     type: ClusterIP
   EOF
   ```

4. **Apply the Kubernetes manifests in order:**
   ```bash
   # Apply managed certificate
   kubectl apply -f kubernetes/managed-cert.yml --context=your-gke-context
   
   # Apply deployment
   kubectl apply -f kubernetes/prover.yml --context=your-gke-context
   
   # Apply ingress
   kubectl apply -f kubernetes/ingress.yml --context=your-gke-context
   ```

5. **Update deployment strategy for ReadWriteOnce volumes:**
   ```bash
   kubectl patch deployment k8s-gpu-prover -n sdk --context=your-gke-context \
     --type='merge' -p '{"spec":{"strategy":{"type":"Recreate","rollingUpdate":null}}}'
   ```

### Verification

1. **Check deployment status:**
   ```bash
   kubectl get pods -n sdk --context=your-gke-context -l app=prover
   ```

2. **Check managed certificate status:**
   ```bash
   kubectl get managedcertificate -n sdk --context=your-gke-context
   ```

3. **Check ingress IP:**
   ```bash
   kubectl get ingress -n sdk --context=your-gke-context
   ```

4. **Test the API:**
   ```bash
   curl -X POST https://your-domain.com/api/prove \
     -H "x-api-key: your-api-key" \
     -H "Content-Type: application/json" \
     -d '{"blueprintId": "test", "proofId": "test"}'
   ```

### Important Notes

- The deployment uses `Recreate` strategy due to ReadWriteOnce PVC constraints
- Google managed certificates can take 10-60 minutes to provision
- Ensure your domain DNS points to the static IP before certificate provisioning
- The deployment requires GPU nodes with NVIDIA Tesla T4 or compatible GPUs

## Error Handling

The service returns appropriate HTTP status codes with error messages:

- `400 Bad Request`: Invalid input parameters
- `401 Unauthorized`: Invalid or missing API key
- `500 Internal Server Error`: Server-side errors during proof generation

## License

This project is part of the ZK Email ecosystem. See the license file for details.
