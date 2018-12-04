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
    id: String,

}

fn main() {
    let mut foo = Foo::default();
    foo.set_private(1);
    (*foo.private_mut()) += 1;
    assert_eq!(*foo.private(), 2);
}
```
*/

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use syn::{Attribute, DataStruct, DeriveInput, Field};

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
}

impl Builder {
    pub fn new(ast: &DeriveInput) -> Builder {
        Builder {
            fields: Vec::new(),
            id: None,
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
                let idx_attr = find_attr_name(f, "index").is_some();
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
            self.id = Some(field.clone())
        }
    }

    fn gen_methods(&self) -> Vec<TokenStream2> {
  
        let mut fieldvec = self.fields.iter().map(|f| {
            format!("\"{}\",", f.clone().ident.unwrap())
        }).collect::<Vec<String>>();
        fieldvec.insert(0, "[".into());
        fieldvec.push("]".into());
        
        let mut fieldtuples = self.fields.iter().map(|f| {
            let field = f.clone().ident.unwrap();
            format!("\"({}, self.{}.as_bytes())\",", field, field)
        }).collect::<Vec<String>>();
        fieldtuples.insert(0, "[".into());
        fieldtuples.push("]".into());

        let mut streams =
            self.fields.iter().map(|field| {
                let field_name = field
                    .clone()
                    .ident
                    .expect("Expected the field to have a name");

                let mdex = Ident::new(
                    &format!("Megadex<{}>", self.typename.clone()),
                    Span::call_site());

                let fn_find_by = Ident::new(
                    &format!("find_by_{}", field_name),
                    Span::call_site(),
                );
             
                let fn_id_by = Ident::new(
                    &format!("id_by_{}", field_name),
                    Span::call_site(),
                );

                let ty = field.ty.clone();

                quote! {
                    pub fn #fn_find_by(md: &#mdex, field: &#ty) -> Result<Vec<Self>, MegadexError> {
                        md.get_by_field(#field_name, field.as_bytes())
                    }

                    pub fn #fn_id_by(md: &#mdex, key: &#ty) -> Result<Vec<&[u8]>, MegadexError> {
                        let envlock = mdf.get_env().read()?;
                        let reader = envlock.read_multi()?;
                        let mditer = md.get_ids_by_field(&reader, #field_name, key.as_bytes())?;
                        Ok(mditer.map(|iter| iter.collect::<Vec<&[u8]>>()))
                    }

                }
            })
            .collect::<Vec<TokenStream2>>();

        let id = self.id.as_ref().expect("At least 1 id attribute field must be specified");
        let field_name = id
            .clone()
            .ident
            .expect("Expected the field to have a name");
        
        let mytype = self.typename.clone();

        let mdex = Ident::new(
            &format!("Megadex<{}>", &mytype),
            Span::call_site());

        let ty = id.ty.clone();

        let str = 
            quote! {
                pub  fn init(db: Db) -> Result<#mdex, MegadexError> {
                    Megadex::new(db, #(#fieldvec)*)
                }

                pub fn save(&self, md: &#mdex) -> Result<(), MegadexError> {
                    md.put(md, self.id.as_bytes(), self, #(#fieldtuples)*)
                }

                pub fn erase(&self, md: &#mdex) -> Result<(), MegadexError> {
                    md.del(md, self.id.as_bytes(), self, #(#fieldtuples)*)
                }

                pub fn get(md: &#mdex, id: &#ty) -> Result<Option<Self>, MegadexError> {
                    md.get(id.as_bytes()) 
                }

                pub fn del(md: &#mdex, id: &#ty, val: &#mytype) -> Result<(), MegadexError> {
                    md.del(md, id.as_bytes(), &bincode::serialize(val)?.map_err(e.into()), #(#fieldtuples)*)
                }

                pub fn insert(md: &#mdex, id: &#ty, val: &#mytype) -> Result<(), MegadexError> {
                    md.put(md, id.as_bytes(), &bincode::serialize(val)?.map_err(e.into()), #(#fieldtuples)*)
                }
            };

        streams.push(str);

        streams
    }
}
