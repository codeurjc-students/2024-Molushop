# Imagen de Molushop para desplegar (Render u otro sitio con Docker).
#
#   docker build -t molushop .
#   docker run --rm -p 8080:8080 --env-file .env.docker molushop
#
# Dos fases: se compila con la imagen completa de Rust y solo el binario y `ui/` pasan
# a una Debian mínima. Todo se hace en release: `/ui` y `/sandbox` no existen en esta
# imagen y `APP_ENV=dev` no arranca (ver src/app_env.rs).

# ---------- Fase 1: compilar ----------
FROM rust:1.99-trixie AS builder

# diesel con la feature "postgres" enlaza con libpq (pq-sys), aunque las consultas
# vayan por tokio-postgres (diesel-async).
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Primero solo las dependencias, con un main vacío: mientras Cargo.toml y Cargo.lock
# no cambien, Docker reutiliza esta capa y no las vuelve a compilar.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

# Ahora el código de verdad. askama lee las plantillas de `ui/` al compilar y
# fluent-templates mete `locales/` dentro del binario.
COPY askama.toml ./
COPY ui ./ui
COPY locales ./locales
COPY src ./src
# touch: que cargo vea main.rs más nuevo que el del paso anterior y lo recompile.
RUN touch src/main.rs && cargo build --release --locked

# ---------- Fase 2: ejecutar ----------
FROM debian:trixie-slim

# libpq5: la necesita el binario (ver arriba).
# ca-certificates: el SDK de AWS (S3) valida con los certificados del sistema.
# (Postgres/Neon no: lleva sus raíces dentro del binario con webpki-roots.)
RUN apt-get update \
    && apt-get install -y --no-install-recommends libpq5 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Sin root: si alguien consiguiera ejecutar algo dentro, no sería administrador.
RUN useradd --system --no-create-home --shell /usr/sbin/nologin molushop

WORKDIR /app
COPY --from=builder /app/target/release/Molushop /app/molushop
# En ejecución se sirven los estáticos de `ui/` (CSS, JS, imágenes); src/config.rs
# recorre `./ui` al arrancar, por eso WORKDIR es /app.
COPY --chown=root:root ui ./ui

USER molushop

# Dentro del contenedor hay que escuchar en todas las interfaces; 127.0.0.1 (el valor
# por defecto) no se vería desde fuera. APP_ENV no se fija aquí: sin ella es `prod`,
# lo más estricto; Render pone `pre` mientras no haya email.
ENV BIND_ADDR=0.0.0.0:8080
EXPOSE 8080

CMD ["/app/molushop"]
