/*!
Getset, we're ready to go!

A procedural macro for generating the most basic getters and setters on fields.

Getters are generated as `fn field(&self) -> &type`, while setters are generated as `fn field(&mut self, val: type)`.

These macros are not intended to be used on fields which require custom logic inside of their setters and getters. Just write your own in that case!

```rust
#[macro_use]
extern crate getset;

#[derive(Getters, Setters, MutGetters, Default)]
pub struct Foo<T> where T: Copy + Clone + Default {
    /// Doc comments are supported!
    /// Multiline, even.
    #[get] #[set] #[get_mut]
    private: T,

    /// Doc comments are supported!
    /// Multiline, even.
    #[get = "pub"] #[set = "pub"] #[get_mut = "pub"]
    public: T,
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
use syn::{DataStruct, DeriveInput, Field, Attribute, Lit, Meta, MetaNameValue};
use proc_macro2::{Ident, Span};
use proc_macro2::TokenStream as TokenStream2;

fn find_attr_name(field: &Field, name: &str) -> bool {
    field
        .attrs
        .iter()
        .find(|a| { 
            a.interpret_meta()
                .map(|v| v.name())
                .expect("no name for attribute?") == name
        })
}

#[proc_macro_derive(Megadex, attributes(indexed, id))]
pub fn megadex(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast = syn::parse(input).expect("Couldn't parse for getters");

    // Build the impl
    let gen = produce(&ast, |f| {
        let id_attr = find_attr_name("id");
        let idx_attr = find_attr_name("index");
        if id_attr {
            handle_id(f)
        } else if idx_attr {
            handle_indexed(f)
        } else {
            quote!{}
        }
    });
    // Return the generated impl
    gen.into()
}

fn produce(ast: &DeriveInput, worker: fn(&Field) -> TokenStream2) -> TokenStream2 {
    let name = &ast.ident;
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let impl_self = fields.iter().map(worker).collect::<Vec<_>>();
        let impl_self = fields.iter().map(worker).collect::<Vec<_>>();

        quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                #(#generated)*
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        panic!("#[derive(Megadex)] is only defined for structs, not for enums!");
    }
}

fn handle_id(field: &Field) -> TokenStream2 {
}

fn handle_indexed(field: &Field) -> TokenStream2 {

    let field_name = field
        .clone()
        .ident
        .expect("Expected the field to have a name");

    let fn_name1 = Ident::new(
        &format!("get_by_{}", field_name),
        Span::call_site(),
    );
    
    let fn_name2 = Ident::new(
        &format!("id_by_{}", field_name),
        Span::call_site(),
    );

    let ty = field.ty.clone();

    let doc = field
        .attrs
        .iter()
        .filter(|v| attr_name(v).expect("attribute") == "doc")
        .collect::<Vec<_>>();

    quote! {
        #(#doc)*
        #[inline(always)]
        fn #fn_name1(key: &#ty) -> Self {
            &self.#field_name
        }

        #(#doc)*
        #[inline(always)]
        fn #fn_name2(key: &#ty) -> Vec<u8> {
            &self.#field_name
        }

    }
}
