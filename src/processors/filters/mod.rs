use tera::Tera;

pub fn register_all(tera: &mut Tera) {
    tera_text_filters::register_all(tera);
}
