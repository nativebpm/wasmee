#!/bin/bash
set -euo pipefail

ENV=${1:-"prod"}
echo "Deploying to environment: ${ENV}"

REGION="us-central1"
REPO_NAME="nativebpm-repo"
PROJECT_ID="nativebpm"

if [ "$ENV" = "test" ]; then
    SERVICE_NAME="wasmee-site-test"
else
    SERVICE_NAME="wasmee-site"
fi

# 1. Decode GCP service key
if [ -f "gcp-key.json" ]; then
    echo "Using existing gcp-key.json"
elif [ -n "${GCP_SERVICE_KEY:-}" ]; then
    if echo "$GCP_SERVICE_KEY" | grep -q '{'; then
        echo "$GCP_SERVICE_KEY" > gcp-key.json
    else
        echo "$GCP_SERVICE_KEY" | base64 -d > gcp-key.json
    fi
else
    echo "Error: GCP_SERVICE_KEY or gcp-key.json not found."
    exit 1
fi

# 2. Authenticate
gcloud auth activate-service-account --key-file=gcp-key.json
gcloud config set project "$PROJECT_ID"

# 3. Submit build async and poll
IMAGE_TAG="${REGION}-docker.pkg.dev/${PROJECT_ID}/${REPO_NAME}/${SERVICE_NAME}:latest"
echo "Submitting Cloud Build for image: $IMAGE_TAG"
BUILD_ID=$(gcloud builds submit --gcs-source-staging-dir="gs://nativebpm-build-sources-133825711702/source" --tag "$IMAGE_TAG" site --async --format="value(id)")
echo "Submitted build ${BUILD_ID}. Waiting for completion..."

while true; do
    STATUS=$(gcloud builds describe "$BUILD_ID" --format="value(status)" 2>/dev/null || echo "PENDING")
    echo "Current build status: ${STATUS}"
    if [ "$STATUS" = "SUCCESS" ]; then
        echo "Build completed successfully!"
        break
    elif [ "$STATUS" = "FAILURE" ] || [ "$STATUS" = "INTERNAL_ERROR" ] || [ "$STATUS" = "TIMEOUT" ] || [ "$STATUS" = "CANCELLED" ]; then
        echo "Build failed with status: ${STATUS}"
        exit 1
    fi
    sleep 10
done

# 4. Deploy to Cloud Run
gcloud run deploy "$SERVICE_NAME" \
    --image="$IMAGE_TAG" \
    --region="$REGION" \
    --platform=managed \
    --allow-unauthenticated \
    --cpu=0.25 \
    --memory=512Mi \
    --min-instances=0 \
    --max-instances=2 \
    --cpu-throttling

echo "WASMEE Site successfully deployed to $SERVICE_NAME!"
