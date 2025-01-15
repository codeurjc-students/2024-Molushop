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
drop table discounts cascade;
drop table product_seller cascade;
drop table product_base_indentifiers cascade;
drop table product_variations_identifiers cascade;

DROP TRIGGER IF EXISTS set_depth_before_insert_update ON Category;
DROP FUNCTION IF EXISTS calculate_depth();

DROP FUNCTION IF EXISTS update_is_parent() cascade;
DROP TRIGGER IF EXISTS update_is_parent_trigger ON Category;