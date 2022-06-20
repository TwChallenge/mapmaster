#!/bin/bash
base_dir=`dirname "${BASH_SOURCE[0]}"`
mkdir -p openapi
wget http://127.0.0.1:8000/mapmaster/openapi.json -O "$base_dir"/openapi/openapi.json
