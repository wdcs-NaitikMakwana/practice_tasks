#![allow(dead_code)]

use std::future::Future;
use futures::executor::block_on;

fn foo() -> impl Future<Output = ()> {
    async {

    }
}

async fn fooo()  {}

//above both fn declaration has same meaning, both are equavalent.

async fn async_printer() {
    println!("Hello bhailog");
}

async fn some_other_async() { 
    // let res =  async_printer();
    // block_on(res);
    async_printer().await; //have to await on async function in order to run it
    println!("Printing from some other async function after running async printer");
}

fn main() {
    println!("Hello, world!");
   let res = some_other_async();
   block_on(res);
}
