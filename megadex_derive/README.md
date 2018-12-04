# Megadex Derive

[![Build Status](https://travis-ci.org/Hoverbear/getset.svg?branch=master)](https://travis-ci.org/Hoverbear/getset)
[![Build status](https://ci.appveyor.com/api/projects/status/w8v2poyjwsy5d05k?svg=true)](https://ci.appveyor.com/project/Hoverbear/getset)

A procedural macro which provides a `Derive` trait called `Megadex`

This provides the use of two field-level attributes: `#[id]` and `#[indexed]` 

You use `id` to indicate which field in your struct contains the primary key. 

You use `indexed` to indicate any fields by which you want to retrieve this (and other) struct(s)

### Usage: 
```rust

#[derive(Megadex)]
struct Foo {
  #[id]
  my_id: String,
  weeee: String,
  #[indexed]
  bar: String,

  #[indexed]
  baz: String,
}

// This would add these methods to your struct: 

fn save(&self)

fn insert(id, other)

fn erase(&self)

fn del(id, other) 

fn get(id, other)

fn find_by_bar(key: &String)

fn id_by_bar(key: &String)

fn find_by_baz(key: &String)

fn id_by_bar(key: &String)

```
