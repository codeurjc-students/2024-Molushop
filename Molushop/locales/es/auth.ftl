# Identificarse y crear cuenta (modal de login), cerrar sesión, avisos.

## Modal de login (login_base, login_modal)
auth-login-title = Iniciar sesión
auth-register-title = Crear cuenta
auth-username = Usuario
auth-username-placeholder = Tu nombre de usuario
auth-email = Email
auth-email-placeholder = tu@email.com
auth-password = Contraseña
auth-password-placeholder = Tu contraseña
auth-password-repeat = Repite la contraseña
auth-login-submit = Entrar
auth-register-submit = Crear cuenta
auth-forgot = ¿Has olvidado tu contraseña?
auth-remember = Recordarme
auth-to-register = ¿No tienes cuenta? Regístrate
auth-to-login = ¿Ya tienes cuenta? Identifícate
auth-cancel = Cancelar
auth-session-ended = Tu sesión ha terminado. Identifícate para continuar.

## Respuestas del servidor (login_base_controller)
auth-login-ok = Has iniciado sesión.
# El mismo para usuario inexistente y contraseña mala: así no se puede
# averiguar qué usuarios existen.
auth-login-wrong = Usuario o contraseña incorrectos.
auth-login-failed = No se ha podido iniciar la sesión. Inténtalo de nuevo.
auth-register-invalid = Hay datos con un formato incorrecto. Revísalos e inténtalo de nuevo.
auth-register-ok = Cuenta creada. Ya puedes identificarte.
auth-register-failed = No se ha podido crear la cuenta. Inténtalo de nuevo.
auth-error = Ha habido un problema. Inténtalo de nuevo.
auth-logout-failed = No se ha podido cerrar la sesión.

## Avisos (MessageModal) y límite de intentos (rate_limit)
modal-accept = Aceptar
modal-close = Cerrar
modal-error-reference = Error { $status } · ref. { $reference }
rate-limit-title = Demasiados intentos
rate-limit-wait = Demasiados intentos seguidos. Espera { $minutes ->
    [one] 1 minuto
   *[other] { $minutes } minutos
} y vuelve a intentarlo.
rate-limit-back = Volver
