


![Megadex Logo](images/megadex.png)

[![](http://meritbadge.herokuapp.com/megadex)](https://crates.io/crates/megadex)

##  Welcome to Megadex

Megadex is a simple tool that removes the boilerplate for indexing structs via multiple fields.
Presently it uses [rkv](https://github.com/mozilla/rkv) which uses [lmdb](https://github.com/danburkert/lmdb-rs). 

### This is not meant to be used stand-alone

I mean, you *could*, but it is meant to be used with [megadex_derive](https://github.com/rrichardson/megadex/tree/master/megadex_derive) which does the heavy lifting in removing boilerplate. 

This is the actual underlying DB implementation using Rkv. 


