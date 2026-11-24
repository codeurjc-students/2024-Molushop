#!/usr/bin/env bash
# Prepara Garage la primera vez (se puede repetir: lo que ya existe no se toca).
# Con Garage arrancado (docker compose up), desde WSL en esta carpeta:  bash setup.sh
# Al final escribe las 6 líneas para el .env de Molushop.
set -euo pipefail

BUCKET=molushop-dev
KEY_NAME=molushop-dev
g() { docker exec molushop-garage /garage "$@"; }

# 1. Disposición del clúster: un nodo con 10 GB (es un tope lógico, no reserva disco).
if g layout show | grep -q "No nodes currently have a role"; then
  NODE_ID=$(g node id -q | cut -d@ -f1)
  g layout assign -z local -c 10G "$NODE_ID"
  g layout apply --version 1
fi

# 2. Bucket y clave de acceso.
g bucket info "$BUCKET" >/dev/null 2>&1 || g bucket create "$BUCKET"
g key info "$KEY_NAME" >/dev/null 2>&1 || g key create "$KEY_NAME"
g bucket allow --read --write --owner "$BUCKET" --key "$KEY_NAME"

# 3. Lectura pública por la web de Garage (puerto 3902).
g bucket website --allow "$BUCKET"

# 4. Lo que va en el .env (la clave secreta solo sale aquí).
KEY_INFO=$(g key info "$KEY_NAME" --show-secret)
echo
echo "# --- S3 simulado (Garage) ---"
echo "AWS_ACCESS_KEY_ID=$(echo "$KEY_INFO" | awk '/Key ID:/ {print $3}')"
echo "AWS_SECRET_ACCESS_KEY=$(echo "$KEY_INFO" | awk '/Secret key:/ {print $3}')"
echo "AWS_REGION=garage"
echo "AWS_S3_BUCKET_NAME=$BUCKET"
echo "S3_ENDPOINT_URL=http://127.0.0.1:3900"
echo "S3_PUBLIC_BASE_URL=http://$BUCKET.web.localhost:3902"
