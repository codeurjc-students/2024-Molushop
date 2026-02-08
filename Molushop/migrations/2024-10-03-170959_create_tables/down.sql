-- This file should undo anything in `up.sql`
DROP TABLE base_user cascade;
DROP TABLE Buyer cascade;
DROP TABLE Seller cascade;
DROP TABLE Customer_address cascade;
DROP TABLE Category cascade;
DROP TABLE Category_product cascade;
DROP TABLE Products cascade;
DROP TABLE Product_variations cascade;
DROP TABLE Admins cascade;
drop table product_attributes cascade; 
drop table prices cascade;
drop table price_history cascade;
drop table discounts cascade;
drop table discount_history cascade;
drop table product_seller cascade;
drop table identifiers_base cascade;
drop table identifiers_var cascade;
drop table product_base_indentifiers cascade;
drop table product_variations_identifiers cascade;
drop table user_sessions cascade;
drop table carts cascade;
drop table cart_products cascade;
drop table favorites cascade;
drop table ratings cascade;
drop table product_summary_ratings cascade;
drop table images_product cascade;
drop table images_product_variations cascade;

DROP TRIGGER IF EXISTS set_depth_before_insert_update ON Category;
DROP FUNCTION IF EXISTS calculate_depth();

DROP FUNCTION IF EXISTS update_is_parent() cascade;
DROP TRIGGER IF EXISTS update_is_parent_trigger ON Category;

DROP TRIGGER IF EXISTS price_history_trigger ON prices;
DROP FUNCTION IF EXISTS log_price_changes();

DROP TRIGGER IF EXISTS discount_history_update_end_date ON discount_history;
DROP FUNCTION IF EXISTS update_discount_history_end_date();

DROP TRIGGER IF EXISTS trg_user_sessions_updated_at ON user_sessions;
DROP FUNCTION IF EXISTS update_timestamp();