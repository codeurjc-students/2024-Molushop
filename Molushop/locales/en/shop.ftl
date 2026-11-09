# Shop texts in English. Missing keys fall back to Spanish (locales/es).
# See src/i18n.rs. No HTML: a sentence with bold text or a link is split.

## Common
categories = Categories
products = Products
all-products = All products
see-all-products = See all products

## Header (nav1)
nav-search-placeholder = Search for products or brands
nav-search-label = Search products
nav-search-submit = Search
nav-cart = Cart
nav-profile = Profile
nav-orders = My orders
nav-favorites = My favorites
nav-account = My account
nav-logout = Log out
nav-login = Sign in
nav-register = Sign up

## Search suggestions
suggest-label = Search suggestions
suggest-all = See all results for “{ $q }”
suggest-empty = No products for “{ $q }”

## Footer (footer1) and language selector
footer-tagline = Products from independent shops, in a single cart.
footer-shop = Shop
footer-home = Home
footer-about = About us
footer-sell = Sell on Molushop
lang-label = Language

## Home page
home-page-title = Home | Molushop
home-hero-title = Everything you're looking for, from independent shops
home-hero-text = Buy from several shops at once and pay in a single cart.
home-hero-cta = See products

## Favorites
favorites-add = Save to favorites
favorites-remove = Remove from favorites
favorites-added-toast = Saved to favorites.
favorites-removed-toast = Removed from favorites.
favorites-view = View favorites
favorites-error-gone = This product is no longer for sale.
favorites-error = We couldn't update your favorites. Please try again.
favorites-page-title = My favorites | Molushop
favorites-count = { $count ->
    [one] 1 product
   *[other] { $count } products
}
favorites-login-prompt = Sign in to see your favorites.
favorites-empty = You haven't saved any products yet. Tap the heart on the ones you like to keep them here.
card-no-longer-sold = No longer for sale

## Product cards
card-add-to-cart = Add to cart
card-unavailable = Unavailable
card-load-more = Load more products
# $rating already uses the language's decimal separator ("4.5").
stars-label = { $rating } out of 5 stars

## Search
search-title-query = Results for “{ $q }”
search-title-query-category = “{ $q }” in { $category }
search-breadcrumb-label = Category
search-all-categories = All
search-clear-query = Clear search
search-filter-label = Filter by category
search-empty-title = We couldn't find any products
search-empty-text = Try other words or fewer filters.

## Page not found (404)
notfound-page-title = Page not found
notfound-title = This page doesn't exist
notfound-text = The link may be wrong, or the page may no longer be here.
product-notfound-page-title = Product unavailable
product-notfound-title = This product isn't available
product-notfound-text = It may no longer be on sale, or the link may be wrong.

## About us
about-page-title = About us
about-title = About us
# Each paragraph with bold text comes in three pieces: before, bold and after.
about-p1-start = Welcome to Molushop, your go-to place for
about-p1-strong = unique and special products
about-p1-end = you'll love!
about-p2-start = At Molushop, we believe every object has a story and can add a touch of magic to your everyday life. We are an online shop devoted to finding and offering you a
about-p2-strong = careful selection of items
about-p2-end = that combine quality, originality and style. From home decorations that transform spaces to accessories that bring out your personality, every product in our catalogue is chosen with you in mind.
about-p3-start = Our passion is discovering the pieces you won't find just anywhere: the ones that make you smile and turn a gift into something unforgettable. We work directly with
about-p3-strong = carefully selected artisans and suppliers
about-p3-end = to make sure every purchase at Molushop is a satisfying experience and that you receive products of the highest quality.
about-p4 = At Molushop, we don't just sell products, we create connections. We want to be part of your special moments, help you express your creativity and always offer you something new and exciting.
about-p5 = Thank you for choosing Molushop. We hope you enjoy exploring our shop as much as we enjoyed creating it for you!

## Only for the tests in src/i18n.rs: they check plurals.
i18n-test-count = { $count ->
    [one] { $count } product
   *[other] { $count } products
}
