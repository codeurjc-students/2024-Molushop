-- This file should undo anything in `up.sql`
DROP TABLE base_user cascade;
DROP TABLE Buyer cascade;
DROP TABLE Seller cascade;
DROP TABLE Customer_address cascade;
DROP TABLE Category cascade;
DROP TABLE Category_product cascade;
DROP TABLE Product cascade;
DROP TABLE product_attributes_category cascade;
DROP TABLE Product_attributes cascade;
DROP TABLE Product_attributes_options cascade;
DROP TABLE Product_sku cascade;
DROP TABLE Attribute_option_sku cascade;
DROP TABLE Admins cascade;

DROP TRIGGER IF EXISTS set_depth_before_insert_update ON Category;
DROP FUNCTION IF EXISTS calculate_depth();