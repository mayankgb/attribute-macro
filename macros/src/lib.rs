use core::panic;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Type}; 


#[proc_macro_derive(Serialize)]
pub fn serialize_struct(input: TokenStream) -> TokenStream { 
    let ast: DeriveInput = syn::parse(input).unwrap(); 
    let name = &ast.ident; 

    let serialized_fields = match &ast.data { 
        Data::Struct(data_struct) => { 
            match &data_struct.fields { 
                Fields::Named(fields) => { 
                    let field_serialization = fields.named.iter().map(|field|  {
                        let field_name = &field.ident; 
                        match &field.ty { 
                            Type::Path(t) =>{ 
                                let field_type = t.path.segments.iter().last().unwrap().ident.to_string();

                                if field_type == "u32" { 
                                    quote! {
                                        result.extend_from_slice(&self.#field_name.to_be_bytes());
                                    }
                                }else if field_type == "String" {
                                    let len_ident = format_ident!("{}_len", field_name.as_ref().unwrap());
                                    quote! {
                                        let #len_ident:u32 = self.#field_name.len().try_into().unwrap(); 
                                        result.extend_from_slice(&#len_ident.to_be_bytes());
                                        result.extend_from_slice(self.#field_name.as_bytes());
                                    }
                                }else { 
                                    panic!("Only string and u32 are supported")
                                }

                            }
                            _=> panic!("others types are not supported")
                        }
                     });
                     quote! {
                        #(#field_serialization)*
                     }
                }, 
                _=> panic!("only named fields are supported")
            }
        }, 
        _=> panic!("only struct are supported")
    };

    let generated = quote! {
        impl Serialize for #name { 
             fn serialize(&self) -> Vec<u8>{ 
                let mut result = Vec::new(); 
                #serialized_fields
                result
             } 
        }
    };
    generated.into()

}

#[proc_macro_derive(DeserializeStruct)]
pub fn deserialize_struct(input: TokenStream) -> TokenStream { 
    let ast: DeriveInput = syn::parse(input).unwrap();
    let name = &ast.ident; 

    let (deserialize_fields, deserialize_field_name) = match &ast.data { 
        Data::Struct(data_struct) => { 
            match &data_struct.fields { 
                Fields::Named(fields) => { 

                    let mut deserialized_field_name = Vec::new();
                    let mut deserialize_fields = Vec::new(); 

                    for field in &fields.named {
                        let field_name = &field.ident; 
                        match &field.ty { 
                            Type::Path(t) => {
                                 let field_type = t.path.segments.iter().last().unwrap().ident.to_string();

                                 if field_type == "String" { 
                                    let len_ident = format_ident!("{}_len",field_name.as_ref().unwrap());
                                    let len_bytes = format_ident!("{}_len_bytes", field_name.as_ref().unwrap());
                                    let string_bytes = format_ident!("{}_bytes",  field_name.as_ref().unwrap() );

                                    deserialize_fields.push(quote! {
                                        let #field_name = { 
                                            let start_offset: usize = offset;
                                            let string_len_size: usize = 4;
                                            let end_offset: usize = start_offset + string_len_size;
                                            let #len_bytes: [u8; 4] = base[start_offset..end_offset].try_into().unwrap(); 
                                            let #len_ident: usize = u32::from_be_bytes(#len_bytes).try_into().unwrap();
                                            let string_end_offset: usize = (end_offset + #len_ident).try_into().unwrap();
                                            let #string_bytes = base[end_offset..string_end_offset].to_vec();
                                            offset = offset + 4 + #len_ident; 
                                            String::from_utf8(#string_bytes).unwrap()
                                        };
                                    });
                                    deserialized_field_name.push(quote! {
                                        #field_name
                                    });

                                 }else if field_type == "u32" { 
                                    
                                    deserialize_fields.push(quote! {
                                        let #field_name = { 
                                            let bytes: [u8; 4] = base[offset..(offset+4)].try_into().unwrap();
                                            offset += 4;
                                            u32::from_be_bytes(bytes)
                                        }; 
                                        
                                    });

                                    deserialized_field_name.push(quote! {
                                        #field_name
                                    });

                                 }else { 
                                    panic!("only u32 and string are supported right now")
                                 }
                            }, 

                            _ => panic!("not supported")
                        }
                    }

                    (deserialize_fields, deserialized_field_name)
                }, 
                _ => panic!("only named fields are supported")
            }
        }, 
        _ => panic!("only struct is supported")
    };

    let generated = quote! {
        impl Deserialize for #name { 
            fn deserialize(base: &[u8]) -> Result<Self, Error> { 
                let mut offset = 0;
                #(#deserialize_fields)*
                Ok( #name { 
                    #(#deserialize_field_name,)*
                })

            }
        }
    }; 
    generated.into()

}