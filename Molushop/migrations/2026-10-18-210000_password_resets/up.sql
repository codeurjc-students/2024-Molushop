-- "He olvidado mi contraseña": un enlace de un solo uso por email. Se guarda el
-- SHA-256 del token, nunca el token (igual que las sesiones): con la tabla no se
-- puede cambiar la contraseña de nadie.
CREATE TABLE password_resets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES base_user(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL UNIQUE,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    -- NULL mientras no se ha usado. Pedir otro enlace marca los anteriores como usados.
    used_at TIMESTAMP
);

CREATE INDEX idx_password_resets_user ON password_resets (user_id);
