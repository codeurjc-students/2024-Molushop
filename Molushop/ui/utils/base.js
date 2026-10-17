// Utilidades comunes; layout1 carga este fichero en todas las páginas.

// Importes en el formato de la tienda ("9,00", "12.345,67"): el mismo texto que
// pinta el servidor (constants::money::format_amount y el átomo Price). Úsalo
// siempre que el JS escriba un precio, en vez de toFixed().
const MONEY = {
    formatter: new Intl.NumberFormat("es-ES", {
        minimumFractionDigits: 2,
        maximumFractionDigits: 2
    }),

    // Acepta número o texto ("150.99", que es como llega de la BD).
    format: function(value) {
        let number = Number(value);
        if (!Number.isFinite(number)) { return String(value); }
        // Sin "-0,00" para un negativo que redondea a cero, igual que en Rust.
        if (Math.abs(number) < 0.005) { number = 0; }
        return this.formatter.format(number);
    }
};
