-- `user_sessions.refresh_token_hash` guardaba el token de sesión en claro. Desde
-- ahora se guarda su SHA-256 en hexadecimal (session_service::start_session);
-- aquí se pasan las filas antiguas al mismo formato. Un SHA-256 en hex mide 64.
UPDATE user_sessions
SET refresh_token_hash = encode(sha256(convert_to(refresh_token_hash, 'UTF8')), 'hex')
WHERE length(refresh_token_hash) <> 64;
