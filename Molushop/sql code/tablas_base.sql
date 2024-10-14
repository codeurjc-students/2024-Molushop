CREATE TABLE "cart_item" (
  "id" integer PRIMARY KEY,
  "cart_id" integer,
  "product_id" integer,
  "products_sku_id" integer,
  "quantity" integer,
  "created_at" timestamp,
  "updated_at" timestamp
);

CREATE TABLE "cart" (
  "id" integer PRIMARY KEY,
  "user_id" integer,
  "total" integer,
  "created_at" timestamp,
  "updated_at" timestamp
);

CREATE TABLE "wishlist" (
  "id" integer PRIMARY KEY,
  "user_id" integer,
  "product_id" integer,
  "created_at" timestamp,
  "deleted_at" timestamp
);

CREATE TABLE "users" (
  "id" integer PRIMARY KEY,
  "avatar" varchar,
  "first_name" varchar,
  "last_name" varchar,
  "username" varchar UNIQUE NOT NULL,
  "email" varchar UNIQUE NOT NULL,
  "password" varchar,
  "birth_of_date" date,
  "phone_number" varchar,
  "created_at" timestamp,
  "deleted_at" timestamp
);

CREATE TABLE "addresses" (
  "id" integer PRIMARY KEY,
  "user_id" integer,
  "title" varchar,
  "address_line_1" varchar,
  "address_line_2" varchar,
  "country" varchar,
  "city" varchar,
  "postal_code" varchar,
  "landmark" varchar,
  "phone_number" varchar,
  "created_at" timestamp,
  "deleted_at" timestamp
);

CREATE TABLE "categories" (
  "id" integer PRIMARY KEY,
  "name" varchar,
  "description" varchar,
  "created_at" timestamp,
  "deleted_at" timestamp
);

CREATE TABLE "sub_categories" (
  "id" integer PRIMARY KEY,
  "parent_id" integer,
  "name" varchar,
  "description" varchar,
  "created_at" timestamp,
  "deleted_at" timestamp
);

CREATE TABLE "products" (
  "id" integer PRIMARY KEY,
  "name" varchar,
  "description" varchar,
  "summary" varchar,
  "cover" varchar,
  "category_id" varchar,
  "created_at" timestamp,
  "deleted_at" timestamp
);

CREATE TABLE "skus" (
  "id" integer PRIMARY KEY,
  "product_id" integer,
  "sku" varchar,
  "price" varchar,
  "quantity" integer,
  "created_at" timestamp,
  "deleted_at" timestamp
);

CREATE TABLE product_attributes(
	id text primary key,
	name text,
	"created_at" timestamp,
  	"deleted_at" timestamp
);

CREATE TABLE attribute_options(
	id text primary key,
	att_id text references att(id),
	value text
	"created_at" timestamp,
  	"deleted_at" timestamp
);

CREATE TABLE att_options_sku(
	sku_id text references sku(id),
	att_options_id text references att_options(id)
);


create table cart_item(
	
),

create table cart_item(
	
),

create table cart_item(
	
),
	
CREATE TABLE product(
	id text primary key,
	name text
);



create table skus2(
	id text primary key,
	product_id text,
	quant integer,
	price integer
);






delete from att_options_sku;
delete from sku;
delete from att_options;
delete from product;
delete from att;

drop table sku cascade;
drop table product;
drop table att_options cascade;
drop table att;
drop table att_options_sku;
