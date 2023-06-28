use comrak::ComrakOptions;
use tera::{try_get_value, Filter};

pub struct Markdown;

impl Filter for Markdown {
    fn filter(
        &self,
        value: &tera::Value,
        _args: &std::collections::HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let string_content = try_get_value!("markdown", "value", String, value);
        let html = comrak::markdown_to_html(&string_content, &ComrakOptions::default());

        Ok(tera::Value::String(html))
    }
}
