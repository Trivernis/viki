use tera::Tera;
mod markdown;

pub fn register_all(tera: &mut Tera) {
    tera_text_filters::register_all(tera);
    tera.register_filter("markdown", markdown::Markdown);
}
