create table category_pro(
	id varchar(10) primary key,
	name text,
	parent varchar(10),
	depth integer,
	foreign key (parent) references category_pro(id)
);


INSERT INTO category_pro (id, name, parent, depth)
VALUES
    ('CAT01', 'Categoría Principal 1', NULL, 0),
    ('CAT02', 'Categoría Principal 2', NULL, 0),
    ('SUB01', 'Subcategoría de CAT01', 'CAT01', 1),
    ('SUB02', 'Otra Subcategoría de CAT01', 'CAT01', 1),
    ('SUB03', 'Subcategoría de CAT02', 'CAT02', 1),
    ('SUBSUB01', 'Subsubcategoría de SUB01', 'SUB01', 2);


Select * from category_pro where depth=0;

Select * from category_pro where id='CAT01';

WITH RECURSIVE descendants AS (
  SELECT id, name, parent, depth
  FROM category_pro
  WHERE parent = 'CAT01'
  UNION ALL
  SELECT c.id, c.name, c.parent, c.depth
  FROM category_pro c
  JOIN descendants d ON c.parent = d.id
)
SELECT * FROM descendants;