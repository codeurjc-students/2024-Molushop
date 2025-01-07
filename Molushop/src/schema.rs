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
        name -> Varchar,
        #[max_length = 255]
        lastname -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        #[max_length = 255]
        hash -> Varchar,
        birthdate -> Date,
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
    category (id) {
        #[max_length = 10]
        id -> Varchar,
        name -> Nullable<Text>,
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
    discounts (id) {
        id -> Int4,
        variation_id -> Uuid,
        #[max_length = 15]
        discount_type -> Nullable<Varchar>,
        discount_value -> Nullable<Numeric>,
        start_date -> Nullable<Timestamp>,
        end_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    prices (id) {
        id -> Int4,
        variation_id -> Uuid,
        price -> Nullable<Numeric>,
        #[max_length = 3]
        currency -> Nullable<Bpchar>,
        start_date -> Nullable<Timestamp>,
        end_date -> Nullable<Timestamp>,
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
    product_variations (id) {
        id -> Uuid,
        product_id -> Uuid,
        identifiers -> Nullable<Jsonb>,
        sku -> Nullable<Text>,
        attributes -> Nullable<Jsonb>,
        stock -> Int4,
        images -> Nullable<Jsonb>,
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
        status -> Nullable<Int2>,
        specs -> Jsonb,
        variations -> Nullable<Jsonb>,
        images -> Nullable<Jsonb>,
        published -> Bool,
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

diesel::joinable!(admins -> base_user (id));
diesel::joinable!(buyer -> base_user (id));
diesel::joinable!(category_product -> category (category_id));
diesel::joinable!(category_product -> products (product_id));
diesel::joinable!(customer_address -> base_user (customer_id));
diesel::joinable!(discounts -> product_variations (variation_id));
diesel::joinable!(prices -> product_variations (variation_id));
diesel::joinable!(product_variations -> products (product_id));
diesel::joinable!(seller -> base_user (id));

diesel::allow_tables_to_appear_in_same_query!(
    admins,
    base_user,
    buyer,
    category,
    category_product,
    customer_address,
    discounts,
    prices,
    product_attributes,
    product_variations,
    products,
    seller,
);
