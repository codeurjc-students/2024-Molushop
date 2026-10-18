// Login de /panel/login. Llama al mismo endpoint que el modal de la tienda
// (JSON { username, pass }); si va bien, la cookie ya está puesta y se recarga
// la página: es el servidor quien decide si lleva a `next` (tiene rol) o
// enseña el aviso de "sin acceso" (no lo tiene).
(function () {
    const root = document.querySelector(".panel-login-1");
    if (!root) return;

    const form = root.querySelector(".login-form");
    const error = root.querySelector(".login-error");
    const button = form.querySelector("button[type=submit]");

    function showError(message) {
        error.textContent = message;
        error.hidden = false;
    }

    form.addEventListener("submit", async function (event) {
        event.preventDefault();
        error.hidden = true;
        button.disabled = true;

        try {
            const res = await fetch(root.dataset.login_url, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({
                    username: form.username.value,
                    pass: form.pass.value
                })
            });

            if (res.ok) {
                window.location.reload();
                return;
            }
            // Mismo mensaje para "no existe" y "contraseña mal": no se dice cuál
            if (res.status === 401) {
                showError("Usuario o contraseña incorrectos.");
            } else if (res.status === 429) {
                // Límite de intentos (middleware rate_limit); Retry-After en segundos
                let minutes = Math.max(1, Math.ceil((parseInt(res.headers.get("Retry-After")) || 60) / 60));
                showError(`Demasiados intentos seguidos. Espera ${minutes} ${minutes === 1 ? "minuto" : "minutos"} y vuelve a intentarlo.`);
            } else {
                showError("No se ha podido iniciar sesión. Inténtalo de nuevo.");
            }
        } catch (e) {
            showError("No se ha podido conectar con el servidor.");
        }
        button.disabled = false;
    });
})();
