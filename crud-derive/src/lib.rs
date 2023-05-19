use cruet::Inflector;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Arm, Attribute, Data, DataEnum, DataStruct,
    DeriveInput, Error, Expr, Field, Fields, FieldsNamed, FieldsUnnamed, Ident, Meta, Variant,
};

#[proc_macro_derive(Postable, attributes(postable, serde, serde_with))]
pub fn derive_submitted(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
    let attrs = input
        .attrs
        .into_iter()
        .filter(|a| a.path().is_ident("schemars"));
    let vis = input.vis;
    let input_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let post_name = format_ident!("__{}Post", input_name);
    TokenStream::from(
        if let Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) = input.data
        {
            let idents = named.clone().into_iter().map(|f| f.ident);
            let mut vals = <Vec<Expr>>::new();
            let post_fields = named.into_iter().map(|f| {
                let mut is_super = false;
                let attrs = f
                    .attrs
                    .into_iter()
                    .filter_map(|a| {
                        if ["serde", "serde_with", "schemars"]
                            .into_iter()
                            .any(|i| a.path().is_ident(i))
                        {
                            Some(a)
                        } else {
                            if a.path().is_ident("postable") {
                                if let Meta::List(list) = a.meta {
                                    if let Ok(ident) = list.parse_args::<Ident>() {
                                        if ident == "into" {
                                            is_super = true;
                                            None
                                        } else {
                                            Some(parse_quote!(#list))
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    })
                    .collect();
                let ident = f.ident.clone();
                let ty = f.ty;
                vals.push(if is_super {
                    parse_quote!(#ident.into())
                } else {
                    parse_quote!(#ident)
                });
                let field = Field {
                    attrs,
                    ty: if is_super {
                        parse_quote!(<#ty as Postable>::Post)
                    } else {
                        ty
                    },
                    ..f
                };
                field
            });
            quote! {
                const _: () = {
                    #[derive(::schemars::JsonSchema)]
                    #(#attrs)*
                    #[derive(::serde::Deserialize)]
                    #[serde(default)]
                    #[serde(rename_all(deserialize = "camelCase"))]
                    #[derive(Default)]
                    #vis struct #post_name #impl_generics #where_clause {
                        #(#post_fields),*
                    }

                    impl #impl_generics ::crud::Post for #post_name #ty_generics #where_clause {}

                    impl #impl_generics ::core::convert::From::<#post_name #ty_generics> for #input_name #ty_generics #where_clause {
                        fn from(value: #post_name #ty_generics) -> Self {
                            Self {
                                #(#idents: value.#vals),*
                            }
                        }
                    }

                    impl #impl_generics ::crud::Postable for #input_name #ty_generics #where_clause {
                        type Post = #post_name #ty_generics;
                    }
                };
            }
        } else {
            Error::new(span, "Named Struct Only :)").to_compile_error()
        },
    )
}

#[proc_macro_derive(Patchable, attributes(patchable, serde, serde_with, schemars))]
pub fn derive_patch(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
    let attrs = input
        .attrs
        .into_iter()
        .filter(|a| a.path().is_ident("schemars"));
    let vis = input.vis;
    let input_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let patch_name = format_ident!("__{}Patch", input_name);
    TokenStream::from(
        if let Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) = input.data
        {
            let patch_fields = named
                .into_iter()
                .filter(|f| f.attrs.iter().any(|a| a.meta.path().is_ident("patchable")))
                .map(|f| {
                    let mut is_super = false;
                    let ty = f.ty;
                    let mut attrs: Vec<_> = f
                        .attrs
                        .into_iter()
                        .filter_map(|a| {
                            if a.path().is_ident("patchable") {
                                if let Meta::List(list) = a.meta {
                                    if let Ok(ident) = list.parse_args::<Ident>() {
                                        if ident == "into" {
                                            is_super = true;
                                            None
                                        } else {
                                            Some(parse_quote!(#list))
                                        }
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else if ["serde", "serde_with", "schemars"]
                                .into_iter()
                                .any(|i| a.path().is_ident(i))
                            {
                                Some(a)
                            } else {
                                None
                            }
                        })
                        .collect();
                    if !is_super {
                        attrs.push(parse_quote! {#[serde(
                            default,
                            skip_serializing_if = "::core::option::Option::is_none",
                        )]});
                    }
                    Field {
                        attrs,
                        ty: if is_super {
                            parse_quote!(<#ty as ::crud::Patchable>::Patch)
                        } else {
                            parse_quote!(::core::option::Option<#ty>)
                        },
                        ..f
                    }
                });
            quote! {
                const _: () = {
                    #[derive(::schemars::JsonSchema)]
                    #(#attrs)*
                    //#[::serde_with::skip_serializing_none]
                    #[derive(::serde::Serialize, ::serde::Deserialize)]
                    #[serde(rename_all(deserialize = "camelCase"))]
                    #vis struct #patch_name #impl_generics #where_clause {
                        #(#patch_fields),*
                    }

                    impl #impl_generics ::crud::Patch for #patch_name #ty_generics #where_clause {}

                    impl #impl_generics ::crud::Patchable for #input_name #ty_generics #where_clause {
                        type Patch = #patch_name #ty_generics;
                    }
                };
            }
        } else {
            Error::new(span, "Named Struct Only :)").to_compile_error()
        },
    )
}

fn view_attrs(attrs: impl IntoIterator<Item = Attribute>) -> (Vec<Attribute>, bool) {
    let mut is_super = false;
    (
        attrs
            .into_iter()
            .filter_map(|a| {
                if a.path().is_ident("viewable") {
                    if let Meta::List(list) = a.meta {
                        if let Ok(ident) = list.parse_args::<Ident>() {
                            if ident == "into" {
                                is_super = true;
                                None
                            } else {
                                Some(parse_quote!(#list))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else if ["serde", "serde_with", "schemars"]
                    .into_iter()
                    .any(|i| a.path().is_ident(i))
                {
                    Some(a)
                } else {
                    None
                }
            })
            .collect(),
        is_super,
    )
}

#[proc_macro_derive(Viewable, attributes(viewable, serde, serde_with, schemars))]
pub fn derive_view(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let span = input.span();
    let attrs = input
        .attrs
        .into_iter()
        .filter(|a| a.path().is_ident("schemars"));
    let vis = input.vis;
    let input_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let view_name = format_ident!("__{}View", input_name);
    let derives = quote! {
        #[derive(::schemars::JsonSchema)]
        #(#attrs)*
        #[derive(::serde::Serialize)]
        #[serde(rename_all(serialize = "camelCase"))]
    };
    let trait_implies = quote! {
        impl #impl_generics ::crud::View for #view_name #ty_generics #where_clause {
            type Object = #input_name #ty_generics;
        }

        impl #impl_generics ::crud::Viewable for #input_name #ty_generics #where_clause {
            type View = #view_name #ty_generics;
        }
    };
    TokenStream::from({
        match input.data {
            Data::Struct(DataStruct {
                fields: Fields::Named(FieldsNamed { named, .. }),
                ..
            }) => {
                let view_fields = named
                    .into_iter()
                    .filter(|f| f.attrs.iter().any(|a| a.path().is_ident("viewable")))
                    .map(|f| {
                        let (attrs, is_super) = view_attrs(f.attrs);
                        let ty = f.ty;
                        Field {
                            attrs,
                            ty: if is_super {
                                parse_quote!(<#ty as ::crud::Viewable>::View)
                            } else {
                                ty
                            },
                            ..f
                        }
                    });
                let view_idents = view_fields.clone().map(|f| f.ident);
                quote! {
                    #derives
                        #vis struct #view_name #impl_generics #where_clause {
                            #(#view_fields),*
                        }

                        impl #impl_generics ::core::convert::From::<#input_name #ty_generics> for #view_name #ty_generics #where_clause {
                            fn from(value: #input_name #ty_generics) -> Self {
                                Self {
                                    #(#view_idents: value.#view_idents.into()),*
                                }
                            }
                        }

                        #trait_implies
                }
            }
            Data::Enum(DataEnum { variants, .. }) => {
                let mut arms = <Vec<Arm>>::new();
                let view_variants = variants.into_iter().map(|v| {
                    let ident = &v.ident;
                    Variant {
                        attrs: Vec::new(),
                        fields: match v.fields {
                            Fields::Named(fields_named) => Fields::Named(fields_named),
                            Fields::Unnamed(FieldsUnnamed { paren_token, unnamed }) => {
                                let mut pat_vars = Vec::new();
                                let mut body_vals = <Vec<Expr>>::new();

                                let fields = Fields::Unnamed(FieldsUnnamed { paren_token, unnamed: unnamed.into_iter().enumerate().filter_map(|(u, f)| {
                                    let var_ident = format_ident!("__{}_var_{}", &ident.to_string().to_lowercase(), u);
                                    pat_vars.push(var_ident.clone());
                                    if f.attrs.iter().any(|a| a.path().is_ident("viewable")) {
                                        let (attrs, is_super) = view_attrs(f.attrs);
                                        body_vals.push(if is_super {
                                            parse_quote!(#var_ident.into())
                                        } else {
                                            parse_quote!(#var_ident)
                                        });
                                        let ty = f.ty;
                                        Some(Field {
                                            attrs,
                                            ty: if is_super {
                                                parse_quote!(<#ty as ::crud::Viewable>::View)
                                            } else {
                                                ty
                                            },
                                            ..f
                                        })
                                    } else {
                                        None
                                    }
                                }).collect()});
                                arms.push(parse_quote!(#input_name::#ident(#(#pat_vars),*) => Self::#ident(#(#body_vals),*)));
                                fields
                            }
                            Fields::Unit => {
                                arms.push(parse_quote!(#input_name::#ident => Self::#ident));
                                Fields::Unit
                            }
                        },
                        ..v
                    }

                });
                quote! {
                    const _: () = {
                        #derives
                        #vis enum #view_name #impl_generics #where_clause {
                            #(#view_variants),*
                        }

                        impl #impl_generics ::core::convert::From::<#input_name #ty_generics> for #view_name #ty_generics #where_clause {
                            fn from(value: #input_name #ty_generics) -> Self {
                                match value {
                                    #(#arms),*
                                }
                            }
                        }

                        #trait_implies
                    };
                }
            }
            _ => Error::new(span, "Named Only :)").to_compile_error(),
        }
    })
}
#[proc_macro_derive(Countable)]
pub fn derive_plural(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let input_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let singular_name = format_ident!("__{}_SINGULAR", &input_name.to_string().to_uppercase());
    let singular_value = &input_name.to_string();
    let plural_name = format_ident!("__{}_PLURAL", &input_name.to_string().to_uppercase());
    let plural_value = &input_name.to_string().to_lowercase().to_plural();
    TokenStream::from({
        quote! {
            const _: () = {
                const #singular_name: &str = #singular_value;

                const #plural_name: &str = #plural_value;

                impl #impl_generics ::crud::Countable for #input_name #ty_generics #where_clause {
                    fn singular() -> &'static str {
                        #singular_name
                    }

                    fn plural() -> &'static str {
                        #plural_name
                    }
                }
            };
        }
    })
}
