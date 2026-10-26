# Signing in and creating an account (login modal), logging out, notices.

## Login modal (login_base, login_modal)
auth-login-title = Sign in
auth-register-title = Create account
auth-username = Username
auth-username-placeholder = Your username
auth-email = Email
auth-email-placeholder = you@email.com
auth-password = Password
auth-password-placeholder = Your password
auth-password-repeat = Repeat password
auth-login-submit = Sign in
auth-register-submit = Create account
auth-forgot = Forgot your password?
auth-remember = Remember me
auth-to-register = Don't have an account? Sign up
auth-to-login = Already have an account? Sign in
auth-cancel = Cancel
auth-session-ended = Your session has ended. Sign in to continue.

## Server responses (login_base_controller)
auth-login-ok = You're signed in.
# The same for an unknown user and a wrong password, so nobody can find out
# which users exist.
auth-login-wrong = Wrong username or password.
auth-login-failed = We couldn't sign you in. Please try again.
auth-register-invalid = Some details have the wrong format. Check them and try again.
auth-register-ok = Account created. You can now sign in.
auth-register-failed = We couldn't create your account. Please try again.
auth-error = Something went wrong. Please try again.
auth-logout-failed = We couldn't log you out.

## Notices (MessageModal) and attempt limit (rate_limit)
modal-accept = OK
modal-close = Close
rate-limit-title = Too many attempts
rate-limit-wait = Too many attempts in a row. Wait { $minutes ->
    [one] 1 minute
   *[other] { $minutes } minutes
} and try again.
rate-limit-back = Go back
