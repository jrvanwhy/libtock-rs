extern crate proc_macro;

fn fmt_enum(name: &syn::Ident, data_enum: syn::DataEnum) -> proc_macro2::TokenStream {
    unimplemented!("TODO")
}

fn fmt_union(name: &syn::Ident) -> proc_macro2::TokenStream {
    let mut message = name.to_string();
    message.push_str("{...}");
    let message_bytes = proc_macro2::Literal::byte_string(message.as_bytes());
    syn::parse_quote!(writer.write(#message_bytes))
}

#[proc_macro_derive(TDebug)]
pub fn tdebug_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = syn::parse_macro_input!(input as syn::DeriveInput);

    // The trait impl has to have all the same generics and constraints as the
    // type itself. In addition, we add a `TDebug` bound on all type parameters,
    // so that we can recursively print all values contained in this type.
    let mut input_generics = derive_input.generics;
    for generic_param in input_generics.params.iter_mut() {
        if let syn::GenericParam::Type(ref mut type_param) = generic_param {
            type_param
                .bounds
                .push(syn::parse_quote!(libtock_fmt::TDebug));
        }
    }
    let (impl_generics, ty_generics, where_clause) = input_generics.split_for_impl();

    let name = derive_input.ident;

    let body = match derive_input.data {
        syn::Data::Struct(_) => unimplemented!("TODO"),
        syn::Data::Enum(data_enum) => fmt_enum(&name, data_enum),
        syn::Data::Union(_) => fmt_union(&name),
    };

    quote::quote!(
        impl #impl_generics libtock_fmt::TDebug for #name #ty_generics #where_clause {
            fn fmt<W: libtock_fmt::Writer>(&self, writer: W) -> Result<(), W::Error> {
                #body
            }
        }
    )
    .into()
}
