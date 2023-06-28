use tera::Tera;

mod filters;

pub fn register_all(tera: &mut Tera) {
    filters::register_all(tera);
}
