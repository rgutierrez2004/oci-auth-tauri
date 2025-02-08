#!/bin/bash

# Check if .env file exists
if [ -f .env ]; then
    # Read variables from .env file
    export $(cat .env | grep -v '^#' | xargs)
    
    # Launch the application
    ./src-tauri/target/release/oci-auth-tauri
else
    echo "Error: .env file not found"
    echo "Please create a .env file with OCI_CLIENT_ID and OCI_CLIENT_SECRET"
    exit 1
fi
