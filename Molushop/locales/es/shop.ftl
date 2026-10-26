# Textos de la tienda en español (el idioma por defecto: si a otro idioma le
# falta una clave, sale esta). Ver src/i18n.rs.

## Selector de idioma (pie)
lang-label = Idioma

## Solo para los tests de src/i18n.rs: comprueban los plurales.
i18n-test-count = { $count ->
    [one] { $count } producto
   *[other] { $count } productos
}
