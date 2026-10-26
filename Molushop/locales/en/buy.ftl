# Buying: product page, reviews, cart and checkout (i18n phase 3).

## Atoms: price and quantity
price-unavailable = Unavailable
qty-label = Quantity
qty-decrease = Remove one
qty-increase = Add one

## Product page
product-reviews-count = { $count ->
    [one] 1 review
   *[other] { $count } reviews
}
product-no-reviews = No reviews yet
product-sold-out = Sold out
product-sold-out-msg = This option is sold out right now. Choose another one or come back later.
product-pick-variation = Choose a valid option.
product-variation-unavailable = This option isn't available right now.
product-quantity-positive = The quantity must be greater than 0.

## Reviews
reviews-title = Reviews
reviews-edit = Edit your review
reviews-write = Write a review
reviews-your-rating = Your rating
reviews-stars = { $count ->
    [one] 1 star
   *[other] { $count } stars
}
reviews-comment-label = Comment
reviews-comment-placeholder = Tell us what you thought (optional)
reviews-save = Save changes
reviews-publish = Post review
reviews-buyers-only = Only customers who have received this product can review it.
reviews-login-prompt = Sign in to write a review.
reviews-login-button = Sign in
reviews-empty = This product has no reviews yet.
reviews-average-total = from { $count ->
    [one] 1 review
   *[other] { $count } reviews
}
reviews-mine = Your review
reviews-more = See more reviews ({ $shown } of { $total })
reviews-delete = Delete my review
reviews-delete-confirm = Delete your review? This can't be undone.
reviews-error-no-rating = Choose how many stars you give it.
reviews-error-rating-range = The rating must be from 1 to 5 stars.
reviews-error-too-long = The comment can't be longer than { $max } characters.
reviews-error-product-gone = This product is no longer available.
reviews-error-save = We couldn't save your review. Please try again.
reviews-error-delete = We couldn't delete your review. Please try again.

## Cart
cart-page-title = Cart | Molushop
cart-title = Cart
cart-select-all = Select all items
cart-empty-title = Your cart is empty
cart-empty-text = Add products from the shop and they'll show up here.
cart-line-gone = It's no longer on sale. Remove it from your cart.
cart-summary = Summary
cart-items-total = Items total
cart-discount = Item discount
cart-shipping = Shipping
cart-shipping-free = Free
cart-estimated-total = Estimated total
cart-vat-included = VAT included
cart-continue = Continue
cart-added = Added to your cart.
cart-error-variation = We couldn't find that option.
cart-error-stock = There isn't enough stock.
cart-error-add = We couldn't add it to your cart. Please try again.
cart-error-line-gone = This product is no longer in your cart.
cart-error-update = We couldn't update your cart. Please try again.

## Checkout
checkout-page-title = Checkout | Molushop
checkout-title = Checkout
checkout-empty = There's nothing to buy.
checkout-back-to-cart = Back to cart
checkout-address = Shipping address
checkout-street1 = Address
checkout-street2 = Address line 2
checkout-optional = (optional)
checkout-postal-code = Postcode
checkout-city = City
checkout-province = Province
checkout-items = Items
checkout-confirm = Place order
checkout-sending = Placing order…
checkout-error-address = Some shipping address details are missing.
checkout-error-stock-low = There isn't enough stock left: only { $available } of “{ $product }” remain. Adjust the quantity in your cart.
checkout-error-stock-out = “{ $product }” sold out while you were completing your order.
checkout-error-unavailable = “{ $product }” is no longer available. Remove it from your cart to complete the order. You haven't been charged.
checkout-error-price = One of the items has no price available right now. You haven't been charged.
checkout-error = We couldn't complete your order. You haven't been charged; please try again.
