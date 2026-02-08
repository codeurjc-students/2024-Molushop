// @generated automatically by Diesel CLI.

diesel::table! {
    admins (id) {
        id -> Uuid,
    }
}

diesel::table! {
    base_user (id) {
        id -> Uuid,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        name -> Nullable<Varchar>,
        #[max_length = 255]
        lastname -> Nullable<Varchar>,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        birthdate -> Nullable<Date>,
        #[max_length = 255]
        confirmation_token -> Nullable<Varchar>,
        token_expiration -> Nullable<Timestamp>,
        active -> Nullable<Bool>,
        created -> Nullable<Timestamp>,
        modified -> Nullable<Timestamp>,
    }
}

diesel::table! {
    buyer (id) {
        id -> Uuid,
    }
}

diesel::table! {
    cart_products (id) {
        id -> Uuid,
        cart_id -> Nullable<Uuid>,
        product_var_id -> Nullable<Uuid>,
        quantity -> Int4,
        price_at_time_of_addition -> Nullable<Numeric>,
        added_at -> Timestamptz,
    }
}

diesel::table! {
    carts (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        status -> Int2,
    }
}

diesel::table! {
    category (id) {
        #[max_length = 10]
        id -> Varchar,
        name -> Text,
        #[max_length = 10]
        parent -> Nullable<Varchar>,
        depth -> Nullable<Int4>,
        base_specs -> Nullable<Jsonb>,
        is_parent -> Bool,
    }
}

diesel::table! {
    category_product (category_id, product_id) {
        #[max_length = 10]
        category_id -> Varchar,
        product_id -> Uuid,
    }
}

diesel::table! {
    customer_address (id) {
        id -> Uuid,
        customer_id -> Uuid,
        #[max_length = 255]
        street1 -> Varchar,
        #[max_length = 255]
        street2 -> Nullable<Varchar>,
        #[max_length = 20]
        postal_code -> Varchar,
        #[max_length = 255]
        city -> Varchar,
        #[max_length = 255]
        province -> Varchar,
    }
}

diesel::table! {
    discount_history (id) {
        id -> Int4,
        variation_id -> Uuid,
        discount_type -> Int2,
        percentage -> Nullable<Numeric>,
        quantity -> Nullable<Int4>,
        discount_value -> Numeric,
        #[max_length = 3]
        currency -> Bpchar,
        start_date -> Nullable<Timestamp>,
        end_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
        recorded_at -> Timestamp,
    }
}

diesel::table! {
    discounts (id) {
        id -> Int4,
        variation_id -> Uuid,
        discount_type -> Int2,
        percentage -> Nullable<Numeric>,
        quantity -> Nullable<Int4>,
        discount_value -> Numeric,
        #[max_length = 3]
        currency -> Bpchar,
        start_date -> Nullable<Timestamp>,
        end_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    favorites (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        product_var_id -> Nullable<Uuid>,
        added_at -> Timestamptz,
        #[max_length = 255]
        notes -> Varchar,
    }
}

diesel::table! {
    identifiers_base (id) {
        id -> Int4,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    identifiers_var (id) {
        id -> Int4,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    images_product (id) {
        id -> Uuid,
        product_id -> Uuid,
        image_url -> Text,
        is_main -> Nullable<Bool>,
        display_order -> Int4,
    }
}

diesel::table! {
    images_product_variations (variation_id, image_id) {
        image_id -> Uuid,
        variation_id -> Uuid,
    }
}

diesel::table! {
    price_history (id) {
        id -> Int4,
        variation_id -> Uuid,
        price -> Numeric,
        #[max_length = 3]
        currency -> Bpchar,
        start_date -> Timestamp,
        end_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    prices (id) {
        id -> Int4,
        variation_id -> Uuid,
        price -> Numeric,
        #[max_length = 3]
        currency -> Bpchar,
        start_date -> Timestamp,
    }
}

diesel::table! {
    product_attributes (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    product_base_indentifiers (id) {
        id -> Int4,
        product_id -> Uuid,
        #[max_length = 255]
        identifier -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    product_seller (product_id, seller_id) {
        product_id -> Uuid,
        seller_id -> Uuid,
    }
}

diesel::table! {
    product_summary_ratings (product_id) {
        product_id -> Uuid,
        star_1_count -> Int4,
        star_2_count -> Int4,
        star_3_count -> Int4,
        star_4_count -> Int4,
        star_5_count -> Int4,
        total_reviews -> Int4,
        average_rating -> Numeric,
    }
}

diesel::table! {
    product_variations (id) {
        id -> Uuid,
        product_id -> Uuid,
        identifiers -> Nullable<Jsonb>,
        sku -> Nullable<Text>,
        attributes -> Nullable<Jsonb>,
        status -> Int2,
        stock -> Int4,
        images -> Nullable<Jsonb>,
    }
}

diesel::table! {
    product_variations_identifiers (id) {
        id -> Int4,
        product_variation_id -> Uuid,
        #[max_length = 255]
        identifier -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    products (id) {
        id -> Uuid,
        #[max_length = 100]
        code -> Varchar,
        #[max_length = 100]
        name -> Varchar,
        description -> Text,
        #[max_length = 100]
        brand -> Varchar,
        status -> Int2,
        specs -> Jsonb,
        variations -> Nullable<Jsonb>,
        variation_titles -> Nullable<Jsonb>,
        images -> Nullable<Jsonb>,
        published -> Bool,
    }
}

diesel::table! {
    ratings (rating_id) {
        rating_id -> Int4,
        product_id -> Uuid,
        user_id -> Uuid,
        rating_value -> Int2,
        comment -> Nullable<Text>,
        created_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    seller (id) {
        id -> Uuid,
        rating -> Int4,
        #[max_length = 255]
        store_name -> Varchar,
    }
}

diesel::table! {
    user_sessions (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        jti -> Varchar,
        #[max_length = 512]
        refresh_token_hash -> Varchar,
        issued_at -> Timestamptz,
        expires_at -> Timestamptz,
        last_used_at -> Timestamptz,
        is_revoked -> Bool,
        ip_address -> Nullable<Inet>,
        user_agent -> Nullable<Text>,
        device_info -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(admins -> base_user (id));
diesel::joinable!(buyer -> base_user (id));
diesel::joinable!(cart_products -> carts (cart_id));
diesel::joinable!(cart_products -> product_variations (product_var_id));
diesel::joinable!(carts -> base_user (user_id));
diesel::joinable!(category_product -> category (category_id));
diesel::joinable!(category_product -> products (product_id));
diesel::joinable!(customer_address -> base_user (customer_id));
diesel::joinable!(discount_history -> product_variations (variation_id));
diesel::joinable!(discounts -> product_variations (variation_id));
diesel::joinable!(favorites -> base_user (user_id));
diesel::joinable!(favorites -> product_variations (product_var_id));
diesel::joinable!(images_product -> products (product_id));
diesel::joinable!(images_product_variations -> images_product (image_id));
diesel::joinable!(images_product_variations -> product_variations (variation_id));
diesel::joinable!(price_history -> product_variations (variation_id));
diesel::joinable!(prices -> product_variations (variation_id));
diesel::joinable!(product_base_indentifiers -> products (product_id));
diesel::joinable!(product_seller -> products (product_id));
diesel::joinable!(product_seller -> seller (seller_id));
diesel::joinable!(product_summary_ratings -> products (product_id));
diesel::joinable!(product_variations -> products (product_id));
diesel::joinable!(product_variations_identifiers -> product_variations (product_variation_id));
diesel::joinable!(ratings -> base_user (user_id));
diesel::joinable!(ratings -> products (product_id));
diesel::joinable!(seller -> base_user (id));
diesel::joinable!(user_sessions -> base_user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    admins,
    base_user,
    buyer,
    cart_products,
    carts,
    category,
    category_product,
    customer_address,
    discount_history,
    discounts,
    favorites,
    identifiers_base,
    identifiers_var,
    images_product,
    images_product_variations,
    price_history,
    prices,
    product_attributes,
    product_base_indentifiers,
    product_seller,
    product_summary_ratings,
    product_variations,
    product_variations_identifiers,
    products,
    ratings,
    seller,
    user_sessions,
);
