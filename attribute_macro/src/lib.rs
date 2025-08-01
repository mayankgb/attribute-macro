

use proc_macro::TokenStream; 
use quote::quote; 
use syn::{parse::Parser, punctuated::Punctuated, Data, DeriveInput, Error, Expr, ExprLit, Fields, Lit, Meta, Token}; 

#[proc_macro_derive(MySerde, attributes(serde))]
pub fn serde_json(input: TokenStream) -> TokenStream { 
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident; 

    let serialized_output = match &ast.data { 
        Data::Struct(data_struct) => { 
            match &data_struct.fields {
                Fields::Named(fields) => { 
                    let mut serialized_fields = Vec::new();
                    for field in &fields.named { 
                        let field_ident = field.ident.as_ref().unwrap(); 
                        let field_name_str = field_ident.to_string();
                        let mut rename = None; 
                        let mut skip_if = None; 
                        let mut skip = false; 
                        let mut skip_if_attr = None; 

                        for attr in field.attrs.iter().filter(|a| a.path().is_ident("serde")) { 
                            if let Ok(tokens) = attr.parse_args::<proc_macro2::TokenStream>() { 
                                let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
                                if let Ok(nested) = parser.parse(tokens.into()) { 
                                    for meta in nested { 
                                        match meta { 
                                            Meta::NameValue(nv) if nv.path.is_ident("rename") => {
                                                if let Expr::Lit(ExprLit { 
                                                    lit: Lit::Str(litstr), 
                                                    ..
                                                }) = nv.value { 
                                                    rename = Some(litstr.value());
                                                }
                                            },
                                            Meta::NameValue(nv) if nv.path.is_ident("skip_serializing_if") => {
                                                if let Expr::Lit(ExprLit { 
                                                    lit: Lit::Str(litstr), 
                                                    ..
                                                }) = nv.value { 
                                                    skip_if = Some(litstr.value());
                                                    skip_if_attr = Some(attr)
                                                }
                                            }
                                            Meta::Path(path ) if path.is_ident("skip") => { 
                                                skip = true; 
                                            }

                                            _ => panic!("only skip , skip_if_serialization, rename are supported")
                                        }                    
                                    }
                                }
                            }

                        }

                        if skip { 
                            continue;
                        } 

                        

                        if skip_if.is_some() && rename.is_none() { 
                            return Error::new_spanned(skip_if_attr.unwrap(), "rename must be attribute").to_compile_error().into();
                        }
                        let field_name = rename.as_deref().unwrap_or(&field_name_str);

                       if let Some(func_path_str) = skip_if.clone() { 
                        let tokens: proc_macro2::TokenStream = func_path_str.parse().expect("invalid func path");
                        serialized_fields.push(quote! {
                            if!(#tokens)(&self.#field_ident) { 
                                map.push(format!("\"{}\": {:?}", #field_name, &self.#field_ident.as_ref().unwrap()));
                            }
                        });
                       }else {
                        serialized_fields.push(quote! {
                            map.push(format!("\"{}\": {:?}", #field_name, &self.#field_ident));
                        });
                       }
                    
                    }
                    quote! {
                        #(#serialized_fields)*
                    }
                }, 
                _ => panic!("only named fields are supported")
            }
        }, 
        _ => panic!("only struct is supported")
    };

    let generated = quote! {
        impl #name { 
            fn json(&self) -> String { 
                let mut map: Vec<String> = Vec::new(); 
                #serialized_output
                format!("{{ {} }}", map.join(","))
            }
        }
    };
    generated.into()
}