#!/usr/bin/env bash
# Rehace una base de datos de DESARROLLO desde cero: la borra, la crea con las
# migraciones y la rellena con seeds/init.sql y seeds/dev.sql.
#
#     seeds/reset_dev.sh "postgres://usuario:clave@localhost:5432/molushop_test"
#
# La URL se pasa SIEMPRE a mano (no se lee del .env) para no borrar una base de
# datos por despiste. Se ejecuta desde la carpeta Molushop/ (donde está diesel.toml).
# Necesita diesel-cli y psql. Nunca contra producción: dev.sql mete usuarios con
# una contraseña conocida.
set -euo pipefail

if [ $# -ne 1 ]; then
    echo "Uso: seeds/reset_dev.sh <DATABASE_URL>" >&2
    exit 1
fi
URL="$1"
DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Se va a BORRAR y rehacer: ${URL%%\?*}" | sed -E 's#//([^:]+):[^@]*@#//\1:****@#'

# diesel regenera src/schema.rs al migrar (diesel.toml), y lo haría desde ESTA
# base de datos. Se guarda y se restaura: el reset no debe tocar el código. Si
# sale distinto, la base de datos de la que salió schema.rs no está como dicen
# las migraciones (alguien la cambió a mano): se avisa.
SCHEMA="src/schema.rs"
BACKUP="$(mktemp)"
cp "$SCHEMA" "$BACKUP"
trap 'cp "$BACKUP" "$SCHEMA"; rm -f "$BACKUP"' EXIT

diesel database reset --database-url "$URL"
if ! cmp -s <(tr -d '\r' < "$SCHEMA") <(tr -d '\r' < "$BACKUP"); then
    echo "AVISO: las migraciones generan un schema.rs distinto del actual:" >&2
    diff <(tr -d '\r' < "$BACKUP") <(tr -d '\r' < "$SCHEMA") >&2 || true
    echo "(schema.rs se deja como estaba)" >&2
fi
psql -w -X -q -v ON_ERROR_STOP=1 -f "$DIR/init.sql" "$URL" < /dev/null
psql -w -X -q -v ON_ERROR_STOP=1 -f "$DIR/dev.sql" "$URL" < /dev/null
echo "Hecho: migraciones + init.sql + dev.sql"
