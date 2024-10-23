select * from base_user;
select * from product;
select * from admins;


select * from category;
delete from category;
insert into category values ('1','categoria 1',null);
insert into category values ('1.1','categoria 1.1','1');
insert into category values ('1.1.1','categoria 3','1.1');

insert into category values ('2','categoria 2',null);
insert into category values ('3','categoria 3',null,34);

update category set parent='1.1' where id='3';

CREATE OR REPLACE FUNCTION calculate_depth() 
RETURNS TRIGGER AS $$
DECLARE
    current_parent varchar(10);
    current_depth integer := 0;
BEGIN
    -- Inicializar el valor del parent de la fila que se está insertando o actualizando
    current_parent := NEW.parent;

    -- Subir en la jerarquía contando los niveles hasta que no haya más padres
    WHILE current_parent IS NOT NULL LOOP
        -- Buscar el padre del parent actual
        SELECT parent INTO current_parent
        FROM Category
        WHERE id = current_parent;

        -- Incrementar el contador de profundidad
        current_depth := current_depth + 1;
    END LOOP;

    -- Asignar la profundidad calculada
    NEW.depth := current_depth;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_depth_before_insert_update
BEFORE INSERT OR UPDATE ON Category
FOR EACH ROW
EXECUTE FUNCTION calculate_depth();


DROP TRIGGER IF EXISTS set_depth_before_insert_update ON Category;
DROP FUNCTION IF EXISTS calculate_depth();