# The account: orders, "My account", password recovery and its emails
# (i18n phase 4).

## My orders and an order's page
orders-page-title = My orders | Molushop
order-page-title = Order | Molushop
orders-count = { $count ->
    [one] 1 order
   *[other] { $count } orders
}
orders-login-prompt = Sign in to see your orders.
orders-empty = You haven't placed any orders yet.
orders-keep-shopping = Continue shopping
order-number = Order no. { $number }
order-units = { $count ->
    [one] 1 item
   *[other] { $count } items
}
order-total = Total
order-not-found = We couldn't find that order.
order-go-to-cart = Go to cart
order-confirmed = Order confirmed!
order-items = Items ({ $count })
order-subtotal = Items
order-discount = Discount
order-cancelled = Cancelled
order-status-pending = Pending
order-status-confirmed = Confirmed
order-status-cancelled = Cancelled
order-status-shipped = Shipped
order-status-delivered = Delivered

## My account
account-page-title = My account | Molushop
account-login-prompt = Sign in to change your password or close your sessions.
account-user = Username: { $username }
account-change-password = Change password
account-current-password = Current password
account-new-password = New password
account-repeat-new-password = Repeat the new password
account-password-rules = At least 8 characters, with an uppercase letter, a lowercase letter, a number and a symbol (!@#$%^&*...).
account-password-note = Changing it signs you out on your other devices; you stay signed in on this one.
account-sessions = Sessions
account-no-other-sessions = You're not signed in on any other device.
account-other-sessions = You're also signed in on { $count ->
    [one] 1 other device
   *[other] { $count } other devices
}.
account-close-other-sessions = Sign out of all other devices
account-ok-password = Password changed.
account-ok-sessions-closed = { $count ->
    [one] 1 session on another device has been closed.
   *[other] { $count } sessions on other devices have been closed.
}
account-ok-no-sessions = There were no other open sessions.
account-error-current = The current password is wrong.
account-error-mismatch = The two new passwords don't match.
account-error-weak = The new password isn't secure: at least 8 characters, with an uppercase letter, a lowercase letter, a number and a symbol.
account-error-same = The new password is the same as the current one.

## Password recovery
password-forgot-page-title = Forgot your password | Molushop
password-forgot-sent = If there's an account with that email, we've sent you a link to choose a new password. It expires in { $minutes } minutes.
password-forgot-spam = Not arriving? Check your spam folder or ask again: only the latest link works.
password-forgot-intro = Enter your account email and we'll send you a link to choose a new one.
password-forgot-submit = Send link
password-reset-page-title = New password | Molushop
password-reset-done = Password changed. For security, all your sessions have been closed: sign in with the new one.
password-reset-submit = Save password
password-reset-invalid = This link doesn't work: it has expired, has already been used or is incomplete.
password-reset-new-link = Request another link
password-error-mismatch = The two passwords don't match.
password-error-weak = The password isn't secure: at least 8 characters, with an uppercase letter, a lowercase letter, a number and a symbol.

## Password recovery email (plain text)
mail-reset-subject = Molushop: change your password
mail-reset-body =
    Hi { $username },

    Someone (hopefully you) has asked to change the password of your Molushop account.
    To choose a new one, open this link (it expires in { $minutes } minutes and only works once):

    { $link }

    If you didn't ask for this, ignore this email: your password stays the same.
