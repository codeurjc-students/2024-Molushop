let login_base = {
    // Tras el POST del login (hx-on::after-request, login_base.html): si ha
    // ido bien, se recarga la página ya con la sesión. Si no, el aviso de
    // error ya lo ha pintado htmx.
    do_after_login: function(event) {
        let status = event.detail.xhr.status;
        if (status >= 200 && status < 300) {
            location.reload();
        }
    }
}