CREATE TABLE productos_atributos (
    id_producto integer unique references products(id),
    id_atributo text REFERENCES product_attributes(id)
);

drop table productos_atributos;

select * from products;
select * from product_attributes;

insert into product_attributes values (1,'atributo1',CURRENT_TIMESTAMP);
insert into product_attributes values (2,'atributo2',CURRENT_TIMESTAMP);
insert into product_attributes values (3,'atributo3',CURRENT_TIMESTAMP);

insert into products (id,name,description,summary,cover,category_id,created_at)
values (1,'producto1','producto xd','una mierda','xd','aaa',CURRENT_TIMESTAMP);

insert into products (id,name,description,summary,cover,category_id,created_at)
values (2,'producto2','producto xddd','una mierda2','xddd','aaa',CURRENT_TIMESTAMP);

insert into productos_atributos values (1,1);
insert into productos_atributos values (2,1);

select * from productos_atributos;
