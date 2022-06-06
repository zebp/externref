use syn::NestedMeta;
use anyhow::Result;

#[derive(Debug, Default)]
pub(crate) struct ExternRefOptions {
    pub(crate) name: Option<String>,
}

impl ExternRefOptions {
    /// Parses options for the [externref](crate::externref) macro from metas in the attribute.
    pub fn parse(metas: impl IntoIterator<Item = NestedMeta>) -> Result<Self> {
        let mut options = ExternRefOptions::default();

        for meta in metas.into_iter() {
            let pair = match meta {
                NestedMeta::Meta(syn::Meta::NameValue(pair)) => pair,
                NestedMeta::Meta(_) | NestedMeta::Lit(_) => {
                    anyhow::bail!("Only name value pairs are allowed in this proc-macro")
                }
            };

            let name = pair
                .path
                .get_ident()
                .ok_or_else(|| anyhow::anyhow!("invalid identifier for attribute arguments"))?
                .to_string();
            let value = match pair.lit {
                syn::Lit::Str(lit) => lit.value(),
                _ => anyhow::bail!("Only string literals are valid for externref options"),
            };

            match name.as_ref() {
                "name" => options.name = Some(value),
                x => anyhow::bail!("Invalid option {x}"),
            }
        }

        Ok(options)
    }
}
