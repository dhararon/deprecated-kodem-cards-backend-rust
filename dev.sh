#!/bin/bash
# This script runs the application with hot reloading

# Check if systemfd is installed
if ! command -v systemfd &> /dev/null; then
    echo "systemfd is not installed. Please run 'cargo install systemfd'"
    exit 1
fi

# Check if cargo-watch is installed
if ! command -v cargo-watch &> /dev/null; then
    echo "cargo-watch is not installed. Please run 'cargo install cargo-watch'"
    exit 1
fi

# Set environment variables for development
export DATABASE_URL=${DATABASE_URL:-"postgres://postgres:postgres@localhost:5432/kodem_cards"}
export REDIS_URL=${REDIS_URL:-"redis://localhost:6379"}
export JWT_SECRET=${JWT_SECRET:-"dev_jwt_secret_key_for_development_only"}
export SERVER_PORT=${SERVER_PORT:-"3000"}
export ENVIRONMENT=${ENVIRONMENT:-"development"}

# Firebase Auth environment variables
# Replace these with your actual Firebase project details
export FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID:-"kodemcards"}
export FIREBASE_API_KEY=${FIREBASE_API_KEY:-"api-key"}
export FIREBASE_AUTH_DOMAIN=${FIREBASE_AUTH_DOMAIN:-"api-key"}

# Firebase Emulator configuration
# Set USE_FIREBASE_EMULATOR=true to use the local emulator
export USE_FIREBASE_EMULATOR=${USE_FIREBASE_EMULATOR:-"false"}
export FIREBASE_EMULATOR_HOST=${FIREBASE_EMULATOR_HOST:-"localhost"}
export FIREBASE_AUTH_EMULATOR_PORT=${FIREBASE_AUTH_EMULATOR_PORT:-"9099"}

# Run the application with hot reloading
echo "Starting Kodem Cards backend with hot reloading..."
echo "The application will automatically rebuild and restart when you make changes to the code."
echo "Press Ctrl+C to stop."
echo ""

# Set RUSTFLAGS to allow registering blocking sockets with tokio
export RUSTFLAGS="--cfg tokio_allow_from_blocking_fd"

echo "Using Firebase project: $FIREBASE_PROJECT_ID"
systemfd --no-pid -s http::$SERVER_PORT -- cargo watch -x run
