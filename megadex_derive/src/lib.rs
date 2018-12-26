#![recursion_limit = "128"]
/*!
MEGADEX

A procedural macro for generating routines for storing, retrieving, and indexing structs

Structs that Derive Megadex get routines to create a data store, put, get delete and find_by_fields

All structs must have either an id field, or a member that is tagged with #[id]

If a field is tagged with #[indexed] then it will be able to be used to retrieve the struct
using a generated `find_by_<member>` function

```rust
#[macro_use]
extern crate megadex;

#[derive(Serialize, Deserialize, Megadex)]
pub struct Foo {
    #[id]
    id: String,
    #[indexed]
    foo: String,
}

fn main() {
}
```
*/

// still needs a
extern crate proc_macro;

use syn;
use quote::quote;
use proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{
    Ident,
    Span,
};
use syn::{
    Attribute,
    DataStruct,
    DeriveInput,
    Field,
    LitStr,
    Type,
};

fn find_attr_name<'s>(field: &'s Field, name: &str) -> Option<&'s Attribute> {
    field.attrs.iter().find(|a| a.interpret_meta().map(|v| v.name()).expect("no name for attribute?") == name)
}

#[proc_macro_derive(Megadex, attributes(indexed, id))]
pub fn megadex(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect("Couldn't parse for getters");

    let mut builder = Builder::new(&ast);
    let gen = builder.run(&ast);
    gen.into()
}

struct Builder {
    fields: Vec<Field>,
    id: Option<Field>,
    typename: Ident,
    id_type: Option<Type>,
}

impl Builder {
    pub fn new(ast: &DeriveInput) -> Builder {
        Builder {
            fields: Vec::new(),
            id: None,
            id_type: None,
            typename: ast.ident.clone(),
        }
    }

    pub fn run(&mut self, ast: &DeriveInput) -> TokenStream2 {
        let name = &ast.ident;
        let generics = &ast.generics;
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        // Is it a struct?
        if let syn::Data::Struct(DataStruct {
            ref fields,
            ..
        }) = ast.data
        {
            // let _stock_methods = create_stock(name);
            for f in fields.iter() {
                let id_attr = find_attr_name(f, "id").is_some();
                let idx_attr = find_attr_name(f, "indexed").is_some();
                if id_attr {
                    self.handle_id(f);
                } else if idx_attr {
                    self.handle_indexed(f);
                }
            }

            let impl_self = self.gen_methods();
            quote! {
                impl #impl_generics #name #ty_generics #where_clause {
                    #(#impl_self)*
                }
            }
        } else {
            // Nope. This is an Enum. We cannot handle these!
            panic!("#[derive(Megadex)] is only defined for structs, not for enums!");
        }
    }

    fn handle_indexed(&mut self, field: &Field) {
        self.fields.push(field.clone());
    }

    fn handle_id(&mut self, field: &Field) {
        if let Some(ref id) = self.id {
            panic!(
                "There can only be 1 id field specified per struct. Id is already an attribute on {}",
                id.clone().ident.unwrap()
            );
        } else {
            self.id = Some(field.clone());
            self.id_type = Some(field.clone().ty)
        }
    }

    fn gen_methods(&self) -> Vec<TokenStream2> {
        let fields =
            self.fields.iter().map(|f| {
                let n = f.clone().ident.unwrap().to_string();
                LitStr::new(n.as_str(), Span::call_site())
            }).collect::<Vec<LitStr>>();

        let fieldvec = quote!{ [ #(#fields),* ] };

        let idents_b = self
            .fields
            .iter()
            .map(|f| f.clone().ident.unwrap())
            .collect::<Vec<Ident>>();

        let idents_a = idents_b.clone().into_iter().map(|i|
                LitStr::new(i.to_string().as_str(), Span::call_site())
            ).collect::<Vec<LitStr>>();
        let idents_a1 = idents_a.clone();
        let idents_b1 = idents_b.clone();
        let fieldtuples = quote!{ [ #((#idents_a, &self.#idents_b)),* ] };
        let valtuples = quote!{ [ #((#idents_a1, &&val.#idents_b1)),* ] };
        let valtuples2 = valtuples.clone();
        let fieldtuples2 = fieldtuples.clone();
        let fieldtuples3 = fieldtuples.clone();
        let mut streams = self
            .fields
            .iter()
            .map(|field| {
                let field_name = field.clone().ident.expect("Expected the field to have a name");
                let field_str = LitStr::new(field_name.to_string().as_str(), Span::call_site());
                let typename = self.typename.clone();
                let mdex = quote!{ MegadexDb<#typename> };

                let fn_find_by = Ident::new(&format!("find_by_{}", field_name), Span::call_site());

                let fn_id_by = Ident::new(&format!("id_by_{}", field_name), Span::call_site());
                let id_type = self.id_type.as_ref().unwrap().clone();
                let ty = field.ty.clone();
                quote! {
                    pub fn #fn_find_by(md: &#mdex, field: &#ty) -> Result<Vec<Self>, MegadexDbError> {
                        md.get_by_field(#field_str, field)
                    }

                    pub fn #fn_id_by(md: &#mdex, key: &#ty) -> Result<Vec<#id_type>, MegadexDbError> {
                        let e = md.get_env();
                        let envlock = e.read()?;
                        let reader = envlock.read()?;
                        md.get_ids_by_field(&reader, #field_str, key)
                    }

                }
            })
            .collect::<Vec<TokenStream2>>();

        //panic!(streams.iter().map(|s| s.to_string()).collect::<Vec<String>>().join(" \n "));
        let id = self.id.as_ref().expect("At least 1 id attribute field must be specified");
        let id_name = id.clone().ident.expect("Expected the field to have a name");

        let mytype = self.typename.clone();

        let mdex = quote!{ MegadexDb<#mytype> };

        let ty = id.ty.clone();

        let s = quote! {
            pub  fn init(db: Db) -> Result<#mdex, MegadexDbError> {
                MegadexDb::new(db, &#(#fieldvec)*)
            }

            pub fn save(&self, md: &mut #mdex) -> Result<(), MegadexDbError> {
                md.put(&self.#id_name, self, &#(#fieldtuples2)*)
            }

            pub fn erase(&self, md: &mut #mdex) -> Result<(), MegadexDbError> {
                md.del(&self.#id_name, &#(#fieldtuples3)*)
            }

            pub fn get(md: &#mdex, id: &#ty) -> Result<Option<Self>, MegadexDbError> {
                md.get(id)
            }

            pub fn del(md: &mut #mdex, id: &#ty, val: &#mytype) -> Result<(), MegadexDbError> {
                md.del(&id, &#(#valtuples)*)
            }

            pub fn insert(md: &mut #mdex, id: &#ty, val: &#mytype) -> Result<(), MegadexDbError> {
                md.put(&id, val, &#(#valtuples2)*)
            }
        };

        //panic!(s.to_string());
        streams.push(s);

        streams
    }
}
