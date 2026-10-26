# Shop texts in English. Missing keys fall back to Spanish (locales/es).
# See src/i18n.rs.

## Language selector (footer)
lang-label = Language

## Only for the tests in src/i18n.rs: they check plurals.
i18n-test-count = { $count ->
    [one] { $count } product
   *[other] { $count } products
}
