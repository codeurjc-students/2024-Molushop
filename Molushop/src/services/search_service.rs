//! Búsqueda de la tienda (`/search`): las categorías para filtrar, cada una con
//! cuántos productos a la venta tiene (ella y las que cuelgan de ella) para el
//! texto buscado. Las tarjetas las da `product_card_group_service`.

use diesel::sql_query;
use diesel::sql_types::{Array, BigInt, Nullable, Text};
use diesel::QueryableByName;
use diesel::result::Error;
use diesel_async::RunQueryDsl;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pg::AsyncPgConnection;
type DbPool = Pool<AsyncPgConnection>;

#[derive(QueryableByName, Debug, Clone)]
pub struct CategoryCount {
    #[diesel(sql_type = Text)]
    pub id: String,
    #[diesel(sql_type = Text)]
    pub name: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub parent: Option<String>,
    /// Productos a la venta en ella o por debajo, sin repetir.
    #[diesel(sql_type = BigInt)]
    pub products: i64,
}

/// Todas las categorías con su recuento para `patterns` (los de
/// `CardFilter::like_patterns`; vacío, sin texto). "A la venta" es lo mismo
/// que en `servicesX::get_product_cards1`.
pub async fn category_counts(patterns: &[String], pool: &DbPool) -> Result<Vec<CategoryCount>, Error> {
    let connection = &mut pool.get().await.unwrap();
    sql_query(r#"
        with recursive tree as (
            -- Cada categoría con ella misma y con todas sus descendientes.
            select id, id as root from category
            union all
            select c.id, t.root from category c join tree t on c.parent = t.id
        ),
        visible as (
            select p.id from products p
            join product_seller ps on ps.product_id = p.id
            join seller s on s.id = ps.seller_id and s.status = 1
            where p.status = 1
            and not exists (
                select 1 from unnest($1::text[]) w
                where not (f_unaccent(coalesce(p.name, '')) ilike f_unaccent(w) or f_unaccent(coalesce(p.brand, '')) ilike f_unaccent(w))
            )
        )
        select cat.id::text as id, cat.name, cat.parent::text as parent,
            (select count(distinct v.id)
             from tree t
             join category_product cp on cp.category_id = t.id
             join visible v on v.id = cp.product_id
             where t.root = cat.id) as products
        from category cat
        order by cat.name
    "#)
        .bind::<Array<Text>, _>(patterns)
        .load::<CategoryCount>(connection).await
}

/// De la raíz a `id`, ambas incluidas; vacía si `id` no existe.
pub fn ancestors<'a>(categories: &'a [CategoryCount], id: &str) -> Vec<&'a CategoryCount> {
    let mut path = Vec::new();
    let mut current = categories.iter().find(|c| c.id == id);
    while let Some(category) = current {
        // Una categoría que se tuviera a sí misma por antepasada no colgaría el bucle.
        if path.iter().any(|c: &&CategoryCount| c.id == category.id) { break; }
        path.push(category);
        current = category.parent.as_deref().and_then(|parent| categories.iter().find(|c| c.id == parent));
    }
    path.reverse();
    path
}

/// Las que cuelgan directamente de `parent` (`None`: las de arriba del todo)
/// y tienen algún producto.
pub fn children<'a>(categories: &'a [CategoryCount], parent: Option<&str>) -> Vec<&'a CategoryCount> {
    categories.iter()
        .filter(|c| c.parent.as_deref() == parent && c.products > 0)
        .collect()
}

/// Las que tienen productos y cuyo nombre lleva todas las palabras de `q`,
/// sin contar mayúsculas ni acentos (como `f_unaccent` en las consultas).
/// Para las sugerencias del buscador: las categorías son pocas y se filtran aquí.
pub fn named_like<'a>(categories: &'a [CategoryCount], q: &str) -> Vec<&'a CategoryCount> {
    let words: Vec<String> = q.split_whitespace().map(fold).collect();
    if words.is_empty() { return Vec::new(); }
    categories.iter()
        .filter(|c| c.products > 0)
        .filter(|c| { let name = fold(&c.name); words.iter().all(|w| name.contains(w.as_str())) })
        .collect()
}

/// Minúsculas y sin acentos, para comparar ("Electrónica" -> "electronica").
fn fold(text: &str) -> String {
    text.to_lowercase().chars().map(|c| match c {
        'á' | 'à' | 'â' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        other => other,
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category(id: &str, parent: Option<&str>, products: i64) -> CategoryCount {
        CategoryCount { id: id.into(), name: id.into(), parent: parent.map(Into::into), products }
    }

    #[test]
    fn ancestors_go_from_the_root() {
        let all = vec![category("SPORT", None, 2), category("FITNESS", Some("SPORT"), 1), category("YOGA", Some("FITNESS"), 1)];
        let ids: Vec<_> = ancestors(&all, "YOGA").iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["SPORT", "FITNESS", "YOGA"]);
        assert!(ancestors(&all, "NOPE").is_empty());
    }

    #[test]
    fn children_skip_empty_ones() {
        let all = vec![category("SPORT", None, 2), category("ELEC", None, 0), category("PADEL", Some("SPORT"), 2)];
        let ids: Vec<_> = children(&all, None).iter().map(|c| c.id.as_str()).collect();
        assert_eq!(ids, ["SPORT"]);
    }

    #[test]
    fn named_like_ignores_case_and_accents() {
        let mut all = vec![category("ELEC", None, 3), category("PADEL", Some("SPORT"), 1), category("TV", Some("ELEC"), 0)];
        all[0].name = "Electrónica".into();
        all[1].name = "Pádel".into();
        all[2].name = "Televisores".into();
        let ids = |q: &str| named_like(&all, q).iter().map(|c| c.id.as_str()).collect::<Vec<_>>();
        assert_eq!(ids("ELECTRONICA"), ["ELEC"]);
        assert_eq!(ids("pádel"), ["PADEL"]);
        assert!(ids("tele").is_empty(), "sin productos no sale");
        assert!(ids("   ").is_empty());
    }
}
