// @generated automatically by Diesel CLI.

diesel::table! {
    attribute_option_sku (sku_id, prod_att_option_id) {
        sku_id -> Uuid,
        prod_att_option_id -> Uuid,
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
    ejemplo (id) {
        #[max_length = 10]
        id -> Varchar,
        #[max_length = 50]
        nombre -> Nullable<Varchar>,
        #[max_length = 50]
        apellido -> Nullable<Varchar>,
        edad -> Nullable<Int4>,
    }
}

diesel::table! {
    product (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        description -> Nullable<Text>,
        summary -> Nullable<Text>,
        #[max_length = 255]
        cover -> Nullable<Varchar>,
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
    product_attributes_category (id) {
        id -> Int4,
        product_att_id -> Nullable<Uuid>,
        #[max_length = 10]
        category_id -> Nullable<Varchar>,
    }
}

diesel::table! {
    product_attributes_options (id) {
        id -> Uuid,
        attribute_id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
    }
}

diesel::table! {
    product_sku (id) {
        id -> Uuid,
        product_id -> Uuid,
        #[max_length = 255]
        sku -> Varchar,
        stock -> Int4,
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

diesel::joinable!(attribute_option_sku -> product_attributes_options (prod_att_option_id));
diesel::joinable!(attribute_option_sku -> product_sku (sku_id));
diesel::joinable!(buyer -> base_user (id));
diesel::joinable!(category_product -> category (category_id));
diesel::joinable!(category_product -> product (product_id));
diesel::joinable!(customer_address -> base_user (customer_id));
diesel::joinable!(product_attributes_category -> category (category_id));
diesel::joinable!(product_attributes_category -> product_attributes (product_att_id));
diesel::joinable!(product_attributes_options -> product_attributes (attribute_id));
diesel::joinable!(product_sku -> product (product_id));
diesel::joinable!(seller -> base_user (id));

diesel::allow_tables_to_appear_in_same_query!(
    attribute_option_sku,
    base_user,
    buyer,
    category,
    category_product,
    customer_address,
    ejemplo,
    product,
    product_attributes,
    product_attributes_category,
    product_attributes_options,
    product_sku,
    seller,
);
