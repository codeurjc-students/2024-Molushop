# La cuenta: pedidos, "Mi cuenta", recuperar la contraseña y sus emails
# (fase 4 del i18n).

## Mis pedidos y la ficha de un pedido
orders-page-title = Mis pedidos | Molushop
order-page-title = Pedido | Molushop
orders-count = { $count ->
    [one] 1 pedido
   *[other] { $count } pedidos
}
orders-login-prompt = Identifícate para ver tus pedidos.
orders-empty = Todavía no has hecho ningún pedido.
orders-keep-shopping = Seguir comprando
order-number = Pedido nº { $number }
order-units = { $count ->
    [one] 1 artículo
   *[other] { $count } artículos
}
order-total = Total
order-not-found = No hemos encontrado ese pedido.
order-go-to-cart = Ir a la cesta
order-confirmed = ¡Pedido confirmado!
order-items = Artículos ({ $count })
order-subtotal = Artículos
order-discount = Descuento
order-cancelled = Cancelado
order-status-pending = Pendiente
order-status-confirmed = Confirmado
order-status-cancelled = Cancelado
order-status-shipped = Enviado
order-status-delivered = Entregado

## Mi cuenta
account-page-title = Mi cuenta | Molushop
account-login-prompt = Identifícate para cambiar tu contraseña o cerrar tus sesiones.
account-user = Usuario: { $username }
account-change-password = Cambiar contraseña
account-current-password = Contraseña actual
account-new-password = Nueva contraseña
account-repeat-new-password = Repite la nueva contraseña
account-password-rules = Al menos 8 caracteres, con mayúscula, minúscula, número y un símbolo (!@#$%^&*...).
account-password-note = Al cambiarla se cierran tus sesiones en otros dispositivos; en este sigues dentro.
account-sessions = Sesiones
account-no-other-sessions = No tienes la sesión abierta en ningún otro dispositivo.
account-other-sessions = Tienes la sesión abierta en { $count ->
    [one] 1 dispositivo más
   *[other] { $count } dispositivos más
}.
account-close-other-sessions = Cerrar sesión en todos los demás dispositivos
account-ok-password = Contraseña cambiada.
account-ok-sessions-closed = { $count ->
    [one] Se ha cerrado 1 sesión en otro dispositivo.
   *[other] Se han cerrado { $count } sesiones en otros dispositivos.
}
account-ok-no-sessions = No había otras sesiones abiertas.
account-error-current = La contraseña actual no es correcta.
account-error-mismatch = Las dos contraseñas nuevas no coinciden.
account-error-weak = La nueva contraseña no es segura: al menos 8 caracteres, con mayúscula, minúscula, número y un símbolo.
account-error-same = La nueva contraseña es igual que la actual.

## Recuperar la contraseña
password-forgot-page-title = He olvidado mi contraseña | Molushop
password-forgot-sent = Si hay una cuenta con ese email, te hemos enviado un enlace para elegir una contraseña nueva. Caduca en { $minutes } minutos.
password-forgot-spam = ¿No llega? Mira en la carpeta de spam o vuelve a pedirlo: solo vale el último enlace.
password-forgot-intro = Escribe el email de tu cuenta y te mandaremos un enlace para elegir una nueva.
password-forgot-submit = Enviar enlace
password-reset-page-title = Nueva contraseña | Molushop
password-reset-done = Contraseña cambiada. Por seguridad se han cerrado todas tus sesiones: identifícate con la nueva.
password-reset-submit = Guardar contraseña
password-reset-invalid = Este enlace no vale: ha caducado, ya se usó o no está completo.
password-reset-new-link = Pedir otro enlace
password-error-mismatch = Las dos contraseñas no coinciden.
password-error-weak = La contraseña no es segura: al menos 8 caracteres, con mayúscula, minúscula, número y un símbolo.

## Email de recuperar la contraseña (texto plano)
mail-reset-subject = Molushop: cambia tu contraseña
mail-reset-body =
    Hola, { $username }:

    Alguien (esperamos que tú) ha pedido cambiar la contraseña de tu cuenta de Molushop.
    Para elegir una nueva, abre este enlace (caduca en { $minutes } minutos y solo sirve una vez):

    { $link }

    Si no lo has pedido tú, ignora este email: tu contraseña no cambia.
