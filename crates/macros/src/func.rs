use anyhow::Result;
use proc_macro2::{Span, TokenStream};
use serde::{Deserialize, Serialize};
use syn::{punctuated::Punctuated, token::Comma, *};

use crate::args::ExternRefOptions;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FunctionData {
    /// The name of the function as it appears in the transformed WASM binary.
    pub name: String,
    /// The indicies of arguments that should have the type `externref`.
    pub arg_indicies: Vec<usize>,
    /// If the return type is an `externref`.
    pub ret_is_extern_ref: bool,
}

impl FunctionData {
    /// Parses data necessary for the functions transformation based on it's signature and options.
    pub fn parse<'attrs>(
        sig: &Signature,
        attrs_or_opts: impl Into<AttributesOrOptions<'attrs>>,
    ) -> Result<Self> {
        let attrs_or_opts: AttributesOrOptions<'attrs> = attrs_or_opts.into();
        let opts: ExternRefOptions = attrs_or_opts.try_into()?;
        let name = opts.name.unwrap_or_else(|| sig.ident.to_string());

        let arg_indicies = sig
            .inputs
            .iter()
            .enumerate()
            .filter_map(|(i, arg)| match arg {
                FnArg::Typed(pat_type) if type_is_extern_ref(&pat_type.ty) => Some(i),
                _ => None,
            })
            .collect();

        Ok(Self {
            name,
            arg_indicies,
            ret_is_extern_ref: match &sig.output {
                ReturnType::Type(_, ret_type) => type_is_extern_ref(ret_type),
                _ => false,
            },
        })
    }

    /// Generates a [TokenStream] of a static variable that acts as a custom WASM section
    /// containing information about the function for the transformer.
    pub fn to_data_section_token_stream(&self, module: Option<&str>) -> Result<TokenStream> {
        let fn_name = match module {
            Some(module) => format!("__extern_ref_data_{module}_{}", self.name),
            None => format!("__extern_ref_data_{}", self.name),
        };

        let ident = Ident::new(&fn_name, Span::call_site());

        // The byte representation of the function data encoded into JSON.
        let bytes = serde_json::to_vec(self)?;
        let length = LitInt::new(&bytes.len().to_string(), Span::call_site());

        // Creates a comma separated list of byte literals that are for an array of the JSON bytes.
        let data_byte_str = Lit::ByteStr(LitByteStr::new(&bytes, Span::call_site()));

        Ok(quote::quote! {
            #[allow(incorrect_ident_case)]
            #[allow(clippy::all)]
            #[link_section = #fn_name]
            static #ident: [u8; #length] = *#data_byte_str;
        })
    }
}

// TODO(zeb): support qualified paths and type aliases /somehow/
fn type_is_extern_ref(ty: &Type) -> bool {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .get_ident()
            .map(|ident| *ident == "ExternRef")
            .unwrap_or(false),
        _ => false,
    }
}

pub(crate) enum AttributesOrOptions<'a> {
    Options(ExternRefOptions),
    Attributes(&'a [Attribute]),
}

impl TryInto<ExternRefOptions> for AttributesOrOptions<'_> {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<ExternRefOptions, Self::Error> {
        let attrs = match self {
            AttributesOrOptions::Options(opts) => return Ok(opts),
            AttributesOrOptions::Attributes(attrs) => attrs,
        };

        // Try to find an `externref` attribute and parse those options if found.
        for attr in attrs {
            if let Some(ident) = attr.path.get_ident() {
                if *ident == "externref" {
                    let list: Punctuated<NestedMeta, Comma> =
                        attr.parse_args_with(Punctuated::parse_terminated)?;
                    return ExternRefOptions::parse(list);
                }
            }
        }

        Ok(ExternRefOptions::default())
    }
}

impl From<ExternRefOptions> for AttributesOrOptions<'_> {
    fn from(value: ExternRefOptions) -> Self {
        Self::Options(value)
    }
}

impl<'a> From<&'a [Attribute]> for AttributesOrOptions<'a> {
    fn from(value: &'a [Attribute]) -> Self {
        Self::Attributes(value)
    }
}
