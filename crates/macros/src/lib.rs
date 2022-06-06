//! A crate containing the [macro@externref] macro.
#![forbid(missing_docs)]

mod args;
mod func;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{AttributeArgs, ForeignItem, ForeignItemFn, ItemFn, ItemForeignMod};

use crate::args::ExternRefOptions;
use crate::func::FunctionData;

/// A macro for registering WASM imports/exports that contain `externref`s.
#[proc_macro_attribute]
pub fn externref(args: TokenStream, item: TokenStream) -> TokenStream {
    let args: AttributeArgs = syn::parse_macro_input!(args as AttributeArgs);
    let opts = ExternRefOptions::parse(args).expect("cannot parse macro options");

    let output_stream = if let Ok(ffi_mod) = syn::parse::<ItemForeignMod>(item.clone()) {
        process_foreign_mod(ffi_mod, opts)
    } else if let Ok(func) = syn::parse::<ForeignItemFn>(item.clone()) {
        func.into_token_stream()
    } else if let Ok(func) = syn::parse::<ItemFn>(item) {
        process_fn(func, opts).into_token_stream()
    } else {
        panic!("Not")
    };

    output_stream.into()
}

fn process_foreign_mod(mut ffi_mod: ItemForeignMod, opts: ExternRefOptions) -> TokenStream2 {
    let name = opts.name.expect("extern blocks must have wasm module name");

    ffi_mod.attrs.push(syn::parse_quote! {
        #[link(wasm_import_module = #name)]
    });

    let mut ffi_fn_data = Vec::new();

    for item in &mut ffi_mod.items {
        if let ForeignItem::Fn(func) = item {
            ffi_fn_data.push(process_foreign_fn(func));
        }
    }

    ffi_fn_data
        .into_iter()
        .flat_map(|data| {
            data.to_data_section_token_stream(Some(&name))
                .expect("failed to create data section token stream")
                .into_iter()
        })
        .chain(ffi_mod.into_token_stream().into_iter())
        .collect()
}

fn process_fn(func: ItemFn, opts: ExternRefOptions) -> TokenStream2 {
    let function_data = FunctionData::parse(&func.sig, opts).expect("cannot parse function");
    function_data
        .to_data_section_token_stream(None)
        .expect("failed to create data section token stream")
        .into_iter()
        .chain(func.into_token_stream().into_iter())
        .collect()
}

fn process_foreign_fn(func: &mut ForeignItemFn) -> FunctionData {
    let data =
        FunctionData::parse(&func.sig, func.attrs.as_ref()).expect("failed to parse function data");

    let name = &data.name;
    func.attrs.push(syn::parse_quote! {  #[link_name = #name] });

    data
}
